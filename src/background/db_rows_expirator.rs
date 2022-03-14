use std::{collections::HashMap, sync::Arc};

use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

use crate::{app::AppContext, db::DbRow, db_json_entity::JsonTimeStamp, db_sync::EventSource};

pub struct DbRowsExpirator {
    app: Arc<AppContext>,
}

impl DbRowsExpirator {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for DbRowsExpirator {
    async fn tick(&self) {
        let now = JsonTimeStamp::now();

        let db_tables = self.app.db.get_tables().await;

        for db_table in db_tables {
            let removed_db_rows = db_table.get_expired_rows(now.date_time).await;

            if let Some(removed_db_rows) = removed_db_rows {
                let event_source = EventSource::as_gc();

                let mut persist_moment = DateTimeAsMicroseconds::now();
                persist_moment.add_seconds(1);

                crate::db_operations::write::bulk_delete::execute(
                    self.app.as_ref(),
                    db_table.as_ref(),
                    as_hash_map(removed_db_rows),
                    event_source,
                    &now,
                    persist_moment,
                )
                .await;
            }
        }
    }
}

fn as_hash_map(rows_to_delete: Vec<Arc<DbRow>>) -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();

    for row_to_delete in rows_to_delete {
        if result.contains_key(&row_to_delete.partition_key) {
            result.insert(row_to_delete.partition_key.to_string(), Vec::new());
        }

        result
            .get_mut(row_to_delete.partition_key.as_str())
            .unwrap()
            .push(row_to_delete.row_key.to_string());
    }

    result
}
