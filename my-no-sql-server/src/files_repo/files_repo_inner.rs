use std::collections::BTreeMap;

use ahash::AHashMap;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use crate::persist_repo::{LoadedPartition, LoadedTableAttrs};
use crate::scripts::serializers::table_attrs::TableMetadataFileContract;

use super::size_class::{size_class_for, MIN_SIZE_CLASS};
use super::slot::{decode_slot, encode_slot, slot_bytes_needed, SlotState, SLOT_PREFIX_LEN};

const TABLES_META_FILE: &str = "tables.meta";

/// Where a partition's slot lives: which size-class page-file and which slot
/// index inside it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SlotLocation {
    pub size_class: u32,
    pub slot_index: u64,
}

impl SlotLocation {
    fn offset(&self) -> u64 {
        self.slot_index * self.size_class as u64
    }
}

/// Runtime state of one size-class page-file.
struct ClassState {
    /// Number of slots the file currently holds (file_len / size_class).
    slot_count: u64,
    /// Indices of freed slots available for reuse (the persisted `.delete`).
    free: Vec<u64>,
}

/// The in-memory bookkeeping of the Files backend. Rebuilt entirely by scanning
/// the page-files on `load_all_partitions`, so nothing here needs to be a
/// crash-consistent on-disk structure besides the slots themselves.
pub struct FilesRepoInner {
    root: String,
    classes: AHashMap<u32, ClassState>,
    /// (table_name, partition_key) -> slot location.
    index: AHashMap<(String, String), SlotLocation>,
    /// table_name -> attributes (mirrored to `<root>/tables.meta`).
    tables: BTreeMap<String, TableMetadataFileContract>,
    /// Monotonic per-write counter used as the slot `version` for crash-time
    /// duplicate resolution. Seeded above the max version seen on disk during
    /// the recovery scan, so it stays monotonic across restarts and never
    /// depends on the (non-monotonic) wall clock.
    next_version: u64,
}

/// A slot picked up by the recovery scan.
struct ScannedSlot {
    key: (String, String),
    location: SlotLocation,
    version: u64,
    payload: Vec<u8>,
}

impl FilesRepoInner {
    pub async fn open(root: String) -> Self {
        tokio::fs::create_dir_all(&root)
            .await
            .expect("files_repo: can not create root directory");

        let tables = load_tables_meta(&root).await;
        let classes = discover_class_files(&root).await;

        Self {
            root,
            classes,
            index: AHashMap::new(),
            tables,
            next_version: 0,
        }
    }

    fn class_path(&self, size_class: u32) -> String {
        format!("{}/{}", self.root, size_class)
    }

    fn delete_path(&self, size_class: u32) -> String {
        format!("{}/{}.delete", self.root, size_class)
    }

    // ---- reads / init ----------------------------------------------------

    pub fn get_tables(&self) -> Vec<LoadedTableAttrs> {
        self.tables
            .iter()
            .map(|(table_name, contract)| LoadedTableAttrs {
                table_name: table_name.clone().into(),
                attr: contract.clone().into(),
            })
            .collect()
    }

    /// Scans every page-file, rebuilds the in-memory index and free-lists, and
    /// returns every live partition's compressed payload. After a crash mid
    /// relocation two slots may carry the same key — the higher `version` wins
    /// and the loser is zeroed on disk so it can never resurrect later.
    pub async fn load_all_partitions(&mut self, skip_errors: bool) -> Vec<LoadedPartition> {
        let class_sizes: Vec<u32> = self.classes.keys().copied().collect();

        let mut scanned: Vec<ScannedSlot> = Vec::new();
        let mut max_version: u64 = 0;

        for size_class in class_sizes {
            let slot_count = self.classes.get(&size_class).unwrap().slot_count;
            let path = self.class_path(size_class);
            let file_bytes = tokio::fs::read(&path).await.unwrap_or_default();
            let slot_len = size_class as usize;

            for slot_index in 0..slot_count {
                let start = slot_index as usize * slot_len;
                let end = start + slot_len;
                if end > file_bytes.len() {
                    break;
                }

                match decode_slot(&file_bytes[start..end]) {
                    SlotState::Free => {
                        self.classes
                            .get_mut(&size_class)
                            .unwrap()
                            .free
                            .push(slot_index);
                    }
                    SlotState::Corrupt => {
                        if !skip_errors {
                            panic!(
                                "files_repo: corrupt slot {} in page-file {}",
                                slot_index, path
                            );
                        }
                        println!(
                            "files_repo: skipping corrupt slot {} in {}",
                            slot_index, path
                        );
                        // Reuse the broken slot; it can never be decoded to a key.
                        self.classes
                            .get_mut(&size_class)
                            .unwrap()
                            .free
                            .push(slot_index);
                    }
                    SlotState::Occupied(slot) => {
                        if slot.version > max_version {
                            max_version = slot.version;
                        }
                        scanned.push(ScannedSlot {
                            key: (slot.table_name, slot.partition_key),
                            location: SlotLocation {
                                size_class,
                                slot_index,
                            },
                            version: slot.version,
                            payload: slot.payload,
                        });
                    }
                }
            }
        }

        // Deduplicate by key keeping the highest version; losers are freed.
        let mut winners: AHashMap<(String, String), ScannedSlot> = AHashMap::new();
        let mut losers: Vec<SlotLocation> = Vec::new();

        for slot in scanned {
            match winners.entry(slot.key.clone()) {
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    if slot.version > e.get().version {
                        let old = e.insert(slot);
                        losers.push(old.location);
                    } else {
                        losers.push(slot.location);
                    }
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(slot);
                }
            }
        }

        for location in losers {
            self.zero_slot(location).await;
            self.classes
                .get_mut(&location.size_class)
                .unwrap()
                .free
                .push(location.slot_index);
        }

        // Seed the write counter above every version seen on disk so it stays
        // monotonic across restarts (losers were just freed; winners are kept).
        self.next_version = max_version + 1;

        let mut result = Vec::with_capacity(winners.len());
        for (key, slot) in winners {
            self.index.insert(key.clone(), slot.location);
            result.push(LoadedPartition {
                table_name: key.0,
                partition_key: key.1,
                compressed: slot.payload,
            });
        }

        // Persist the reconciled free-lists so the on-disk `.delete` files match.
        let classes_with_free: Vec<u32> = self
            .classes
            .iter()
            .filter(|(_, state)| !state.free.is_empty())
            .map(|(size_class, _)| *size_class)
            .collect();
        for size_class in classes_with_free {
            self.persist_delete(size_class).await;
        }

        result
    }

    // ---- writes ----------------------------------------------------------

    pub async fn save_partition(&mut self, table_name: &str, partition_key: &str, payload: &[u8]) {
        let needed = slot_bytes_needed(table_name, partition_key, payload.len());
        let size_class = size_class_for(needed);
        let key = (table_name.to_string(), partition_key.to_string());
        let version = self.next_version;
        self.next_version += 1;

        let existing = self.index.get(&key).copied();

        // Synchronous bookkeeping: choose the target slot (in place when the
        // size class is unchanged, otherwise allocate / reuse a free slot).
        let mut delete_file_to_persist: Option<u32> = None;
        let target = match existing {
            Some(location) if location.size_class == size_class => location,
            _ => {
                let reused = self.allocate_slot(size_class);
                if reused.reused_free {
                    delete_file_to_persist = Some(size_class);
                }
                reused.location
            }
        };

        self.index.insert(key, target);

        // Disk writes (no fsync — durability decision).
        let buf = encode_slot(size_class, version, table_name, partition_key, payload);
        self.write_slot(target, &buf).await;

        if let Some(old) = existing {
            if old != target {
                self.zero_slot(old).await;
                self.classes
                    .get_mut(&old.size_class)
                    .unwrap()
                    .free
                    .push(old.slot_index);
                self.persist_delete(old.size_class).await;
            }
        }

        if let Some(size_class) = delete_file_to_persist {
            self.persist_delete(size_class).await;
        }
    }

    pub async fn delete_partition(&mut self, table_name: &str, partition_key: &str) {
        let key = (table_name.to_string(), partition_key.to_string());
        if let Some(location) = self.index.remove(&key) {
            self.zero_slot(location).await;
            self.classes
                .get_mut(&location.size_class)
                .unwrap()
                .free
                .push(location.slot_index);
            self.persist_delete(location.size_class).await;
        }
    }

    pub async fn clean_table_content(&mut self, table_name: &str) {
        let to_free: Vec<((String, String), SlotLocation)> = self
            .index
            .iter()
            .filter(|((t, _), _)| t == table_name)
            .map(|(k, loc)| (k.clone(), *loc))
            .collect();

        let mut touched_classes: Vec<u32> = Vec::new();
        for (key, location) in to_free {
            self.index.remove(&key);
            self.zero_slot(location).await;
            self.classes
                .get_mut(&location.size_class)
                .unwrap()
                .free
                .push(location.slot_index);
            if !touched_classes.contains(&location.size_class) {
                touched_classes.push(location.size_class);
            }
        }

        for size_class in touched_classes {
            self.persist_delete(size_class).await;
        }
    }

    /// Replaces the whole table's content: writes/overwrites every supplied
    /// partition first, then frees only the slots of partitions that are no
    /// longer present. Unlike clean-then-write, this never frees a partition's
    /// slot before its replacement is on disk.
    pub async fn replace_table_partitions(
        &mut self,
        table_name: &str,
        partitions: Vec<(String, Vec<u8>)>,
    ) {
        let new_keys: std::collections::HashSet<&str> =
            partitions.iter().map(|(pk, _)| pk.as_str()).collect();

        for (partition_key, payload) in &partitions {
            self.save_partition(table_name, partition_key, payload)
                .await;
        }

        let stale: Vec<(String, String)> = self
            .index
            .keys()
            .filter(|(t, pk)| t == table_name && !new_keys.contains(pk.as_str()))
            .cloned()
            .collect();

        let mut touched_classes: Vec<u32> = Vec::new();
        for key in stale {
            if let Some(location) = self.index.remove(&key) {
                self.zero_slot(location).await;
                self.classes
                    .get_mut(&location.size_class)
                    .unwrap()
                    .free
                    .push(location.slot_index);
                if !touched_classes.contains(&location.size_class) {
                    touched_classes.push(location.size_class);
                }
            }
        }

        for size_class in touched_classes {
            self.persist_delete(size_class).await;
        }
    }

    pub async fn save_table_metadata(
        &mut self,
        table_name: &str,
        contract: TableMetadataFileContract,
    ) {
        self.tables.insert(table_name.to_string(), contract);
        self.persist_tables_meta().await;
    }

    pub async fn delete_table_metadata(&mut self, table_name: &str) {
        if self.tables.remove(table_name).is_some() {
            self.persist_tables_meta().await;
        }
    }

    /// Reclaims disk: any page-file whose every slot is free (all slot indices
    /// present in the free-list) is fully dead, so both `<size>` and
    /// `<size>.delete` are removed and the class dropped from memory. A future
    /// write to that size class recreates the file from scratch. Partially-free
    /// files are left untouched — their free slots are reused in place. Runs
    /// under the repo mutex, so it never races a write.
    pub async fn vacuum(&mut self) {
        let empty_classes: Vec<u32> = self
            .classes
            .iter()
            .filter(|(_, state)| {
                if state.slot_count == 0 {
                    return false;
                }
                // Every slot index in the file must be in the free-list.
                let free: std::collections::HashSet<u64> = state.free.iter().copied().collect();
                (0..state.slot_count).all(|slot_index| free.contains(&slot_index))
            })
            .map(|(size_class, _)| *size_class)
            .collect();

        for size_class in empty_classes {
            // Remove the page-file first; a stray `.delete` without its page-file
            // is harmless (ignored on discovery). Treat "already gone" as success.
            let removed = match tokio::fs::remove_file(self.class_path(size_class)).await {
                Ok(_) => true,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => true,
                Err(err) => {
                    println!(
                        "files_repo: could not vacuum page-file {}: {} (will retry)",
                        size_class, err
                    );
                    false
                }
            };

            if removed {
                let _ = tokio::fs::remove_file(self.delete_path(size_class)).await;
                self.classes.remove(&size_class);
                println!(
                    "files_repo: vacuumed fully-freed page-file for size class {}",
                    size_class
                );
            }
        }
    }

    // ---- low-level helpers ----------------------------------------------

    /// Reserves a slot index in `size_class`, reusing a freed one when
    /// available, otherwise extending the file by one slot.
    fn allocate_slot(&mut self, size_class: u32) -> AllocResult {
        let state = self.classes.entry(size_class).or_insert(ClassState {
            slot_count: 0,
            free: Vec::new(),
        });

        if let Some(slot_index) = state.free.pop() {
            return AllocResult {
                location: SlotLocation {
                    size_class,
                    slot_index,
                },
                reused_free: true,
            };
        }

        let slot_index = state.slot_count;
        state.slot_count += 1;
        AllocResult {
            location: SlotLocation {
                size_class,
                slot_index,
            },
            reused_free: false,
        }
    }

    async fn write_slot(&self, location: SlotLocation, buf: &[u8]) {
        let path = self.class_path(location.size_class);
        let mut file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            // Never truncate: we overwrite one slot in place and keep the rest
            // of the page-file intact.
            .truncate(false)
            .open(&path)
            .await
            .expect("files_repo: can not open page-file for write");
        file.seek(std::io::SeekFrom::Start(location.offset()))
            .await
            .expect("files_repo: seek failed");
        file.write_all(buf)
            .await
            .expect("files_repo: slot write failed");
    }

    /// Marks a slot free by zeroing its 16-byte prefix (`body_len = 0`), so the
    /// recovery scan treats it as reusable and never reloads its old content.
    async fn zero_slot(&self, location: SlotLocation) {
        let path = self.class_path(location.size_class);
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .await
            .expect("files_repo: can not open page-file to free a slot");
        file.seek(std::io::SeekFrom::Start(location.offset()))
            .await
            .expect("files_repo: seek failed");
        file.write_all(&[0u8; SLOT_PREFIX_LEN])
            .await
            .expect("files_repo: zeroing slot failed");
    }

    async fn persist_delete(&self, size_class: u32) {
        let state = self.classes.get(&size_class).unwrap();
        let mut bytes = Vec::with_capacity(state.free.len() * 8);
        for slot_index in &state.free {
            bytes.extend_from_slice(&slot_index.to_le_bytes());
        }
        atomic_write(&self.delete_path(size_class), &bytes).await;
    }

    async fn persist_tables_meta(&self) {
        let bytes = serde_json::to_vec(&self.tables).unwrap();
        atomic_write(&format!("{}/{}", self.root, TABLES_META_FILE), &bytes).await;
    }
}

struct AllocResult {
    location: SlotLocation,
    reused_free: bool,
}

async fn load_tables_meta(root: &str) -> BTreeMap<String, TableMetadataFileContract> {
    let path = format!("{}/{}", root, TABLES_META_FILE);
    match tokio::fs::read(&path).await {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => BTreeMap::new(),
    }
}

async fn discover_class_files(root: &str) -> AHashMap<u32, ClassState> {
    let mut classes = AHashMap::new();

    let mut read_dir = match tokio::fs::read_dir(root).await {
        Ok(read_dir) => read_dir,
        Err(_) => return classes,
    };

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let Ok(file_type) = entry.file_type().await else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }

        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };

        // Page-files are named by their size class (a plain integer).
        let Ok(size_class) = file_name.parse::<u32>() else {
            continue;
        };
        if size_class < MIN_SIZE_CLASS {
            continue;
        }

        let len = entry.metadata().await.map(|m| m.len()).unwrap_or(0);
        let slot_count = len / size_class as u64;

        classes.insert(
            size_class,
            ClassState {
                slot_count,
                free: Vec::new(),
            },
        );
    }

    classes
}

/// Writes `bytes` to `path` atomically: tmp file -> fsync -> rename. Used for
/// the small metadata files (`tables.meta`, `<S>.delete`) where a torn write
/// would be costly; the slots themselves are deliberately written without fsync.
async fn atomic_write(path: &str, bytes: &[u8]) {
    let tmp_path = format!("{}.tmp", path);
    {
        let mut file = tokio::fs::File::create(&tmp_path)
            .await
            .expect("files_repo: can not create tmp file");
        file.write_all(bytes)
            .await
            .expect("files_repo: tmp write failed");
        file.sync_all().await.expect("files_repo: tmp fsync failed");
    }
    tokio::fs::rename(&tmp_path, path)
        .await
        .expect("files_repo: rename failed");
}
