use my_no_sql_sdk::core::db::{DbTableAttributes, DbTableName};

use crate::files_repo::FilesRepo;
#[cfg(feature = "sqlite")]
use crate::sqlite_repo::SqlLiteRepo;

use super::{LoadedPartition, LoadedTableAttrs};

/// The persistence backend, selected at startup from `PersistenceDest`:
/// a `.sqlite`/`.db` path -> SQLite, any other path -> a directory of
/// slotted page-files (`FilesRepo`). Both store one compressed (zstd) blob
/// per partition. The SQLite variant is a legacy backend, compiled in only
/// with the `sqlite` feature.
pub enum PersistRepo {
    #[cfg(feature = "sqlite")]
    Sqlite(SqlLiteRepo),
    Files(FilesRepo),
}

impl PersistRepo {
    pub async fn save_partition(
        &self,
        table_name: &DbTableName,
        partition_key: &str,
        compressed: &[u8],
    ) {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(repo) => {
                repo.save_partition(table_name.as_str(), partition_key, compressed)
                    .await
            }
            Self::Files(repo) => {
                repo.save_partition(table_name.as_str(), partition_key, compressed)
                    .await
            }
        }
    }

    pub async fn delete_partition(&self, table_name: &DbTableName, partition_key: &str) {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(repo) => {
                repo.delete_partition(table_name.as_str(), partition_key)
                    .await
            }
            Self::Files(repo) => {
                repo.delete_partition(table_name.as_str(), partition_key)
                    .await
            }
        }
    }

    pub async fn clean_table_content(&self, table_name: &DbTableName) {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(repo) => repo.clean_table_content(table_name.as_str()).await,
            Self::Files(repo) => repo.clean_table_content(table_name.as_str()).await,
        }
    }

    pub async fn save_table_metadata(&self, table_name: &DbTableName, attr: &DbTableAttributes) {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(repo) => repo.save_table_metadata(table_name.as_str(), attr).await,
            Self::Files(repo) => repo.save_table_metadata(table_name.as_str(), attr).await,
        }
    }

    pub async fn delete_table_metadata(&self, table_name: &DbTableName) {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(repo) => repo.delete_table_metadata(table_name.as_str()).await,
            Self::Files(repo) => repo.delete_table_metadata(table_name.as_str()).await,
        }
    }

    pub async fn get_tables(&self) -> Vec<LoadedTableAttrs> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(repo) => repo.get_tables().await,
            Self::Files(repo) => repo.get_tables().await,
        }
    }

    pub async fn load_all_partitions(&self, skip_errors: bool) -> Vec<LoadedPartition> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(repo) => repo.load_all_partitions(skip_errors).await,
            Self::Files(repo) => repo.load_all_partitions(skip_errors).await,
        }
    }

    /// Prepares the backend for writes when init skips the normal local-load
    /// path (init-from-other-server). The Files backend must scan its page-files
    /// first — the scan rebuilds the key index and free-lists and seeds the
    /// version counter past every slot already on disk; writing into a non-empty
    /// directory without it would append duplicate slots with LOWER versions,
    /// and the next restart's higher-version-wins dedup would revert the whole
    /// import. SQLite writes are stateless, so this is a no-op there.
    pub async fn prime_for_writes(&self, skip_errors: bool) {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(_) => {}
            Self::Files(repo) => {
                let _ = repo.load_all_partitions(skip_errors).await;
            }
        }
    }

    /// Replaces the entire persisted content of a table with `partitions`
    /// (each `(partition_key, zstd bytes)`), writing the new blobs before
    /// removing any partitions no longer present — so a crash mid-sync cannot
    /// drop a partition that is still part of the table.
    pub async fn replace_table_partitions(
        &self,
        table_name: &DbTableName,
        partitions: Vec<(String, Vec<u8>)>,
    ) {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(repo) => {
                repo.replace_table_partitions(table_name.as_str(), partitions)
                    .await
            }
            Self::Files(repo) => {
                repo.replace_table_partitions(table_name.as_str(), partitions)
                    .await
            }
        }
    }

    pub async fn vacuum(&self) {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(repo) => repo.vacuum().await,
            // Reclaims page-files whose every slot has been freed (partial files
            // keep reusing their free slots in place).
            Self::Files(repo) => repo.vacuum().await,
        }
    }
}
