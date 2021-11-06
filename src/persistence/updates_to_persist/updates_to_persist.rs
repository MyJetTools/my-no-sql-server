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
        let diff = get_table_difference_mut(&mut write_access, table_name);
        diff.partitions_are_updated(partitions, sync_moment);
    }

    pub async fn flag_table_to_update(
        &self,
        table_name: &str,
        sync_moment: DateTimeAsMicroseconds,
    ) {
        let mut write_access = self.data_by_table.lock().await;
        let diff = get_table_difference_mut(&mut write_access, table_name);
        diff.table_is_updated(sync_moment);
    }

    pub async fn update_table_attributes(
        &self,
        table_name: &str,
        sync_moment: DateTimeAsMicroseconds,
    ) {
        let mut write_access = self.data_by_table.lock().await;
        let diff = get_table_difference_mut(&mut write_access, table_name);
        diff.table_attributes_are_updated(sync_moment);
    }

    pub async fn get_next_sync_event(
        &self,
        now: DateTimeAsMicroseconds,
        is_shutting_down: bool,
    ) -> Option<PersistEvent> {
        let mut write_access = self.data_by_table.lock().await;

        for (table_name, table_updates) in &mut *write_access {
            let state = table_updates.get_update_state(now, is_shutting_down);

            if let Some(table_updates_state) = state {
                return Some(PersistEvent {
                    table_name: table_name.to_string(),
                    state: table_updates_state,
                });
            }
        }

        return None;
    }
}

fn get_table_difference_mut<'s>(
    data: &'s mut HashMap<String, TableUpdates>,
    table_name: &str,
) -> &'s mut TableUpdates {
    if !data.contains_key(table_name) {
        data.insert(
            table_name.to_string(),
            TableUpdates::new(table_name.to_string()),
        );
    }

    return data.get_mut(table_name).unwrap();
}
