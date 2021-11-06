use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use super::{table_updates::TableUpdates, TableUpdatesState};

pub struct PersistEvent {
    pub table_name: String,
    pub state: TableUpdatesState,
}

pub struct UpdatesToPersistByTable {
    data_by_table: Mutex<HashMap<String, TableUpdates>>,
}

impl UpdatesToPersistByTable {
    pub fn new() -> Self {
        Self {
            data_by_table: Mutex::new(HashMap::new()),
        }
    }

    pub async fn flag_paritions_to_update<'a, TKeys: Iterator<Item = &'a String>>(
        &self,
        table_name: &str,
        partitions: TKeys,
        sync_moment: DateTimeAsMicroseconds,
    ) {
        let mut write_access = self.data_by_table.lock().await;

        match write_access.get_mut(table_name) {
            Some(update) => {
                update.partitions_are_updated(partitions, sync_moment);
            }
            None => {
                let mut table_update = TableUpdates::new_as_partitions_are_updated(
                    table_name.to_string(),
                    Some(sync_moment),
                );

                table_update.partitions_are_updated(partitions, sync_moment);
                write_access.insert(table_name.to_string(), table_update);
            }
        };
    }

    pub async fn flag_table_to_update(
        &self,
        table_name: &str,
        sync_moment: DateTimeAsMicroseconds,
    ) {
        let mut write_access = self.data_by_table.lock().await;

        match write_access.get_mut(table_name) {
            Some(update) => {
                update.table_is_updated(sync_moment);
            }
            None => {
                let table_update = TableUpdates::new_as_table_is_updated(
                    table_name.to_string(),
                    Some(sync_moment),
                );
                write_access.insert(table_name.to_string(), table_update);
            }
        };
    }

    pub async fn update_table_attributes(
        &self,
        table_name: &str,
        sync_moment: DateTimeAsMicroseconds,
    ) {
        let mut write_access = self.data_by_table.lock().await;

        match write_access.get_mut(table_name) {
            Some(update) => {
                update.table_attributes_are_updated(sync_moment);
            }
            None => {
                let mut table_update = TableUpdates::new_as_partitions_are_updated(
                    table_name.to_string(),
                    Some(sync_moment),
                );

                table_update.table_attributes_are_updated(sync_moment);

                write_access.insert(table_name.to_string(), table_update);
            }
        };
    }

    pub async fn get_next_sync_event(
        &self,
        now: DateTimeAsMicroseconds,
        is_shutting_down: bool,
    ) -> Option<PersistEvent> {
        let mut write_access = self.data_by_table.lock().await;

        let table_name = get_next_key_to_remove(&write_access, now, is_shutting_down);

        if let Some(table_name) = table_name {
            let table_updates = write_access.remove(&table_name);

            if let Some(table_updates) = table_updates {
                return Some(PersistEvent {
                    table_name,
                    state: table_updates.state,
                });
            }
        }

        return None;
    }
}

fn get_next_key_to_remove(
    src: &HashMap<String, TableUpdates>,
    now: DateTimeAsMicroseconds,
    is_shutting_down: bool,
) -> Option<String> {
    for (table_name, update) in src {
        if is_shutting_down {
            return Some(table_name.to_string());
        }

        let common_state = update.get_common_state_data();

        match common_state.sync_moment {
            Some(sync_moment) => {
                if now.unix_microseconds >= sync_moment.unix_microseconds {
                    return Some(table_name.to_string());
                }
            }
            None => return Some(table_name.to_string()),
        }
    }

    None
}
