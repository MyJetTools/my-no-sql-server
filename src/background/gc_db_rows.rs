use std::{collections::HashMap, sync::Arc};

use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::EventSource,
    utils::{LazyHashMap, LazyVec},
};

struct DataToExpire {
    partitions_to_expire: Option<Vec<(Arc<DbTable>, Vec<String>)>>,
    db_rows_to_expire: Option<Vec<(Arc<DbTable>, HashMap<String, Vec<String>>)>>,
}

pub struct GcDbRows {
    app: Arc<AppContext>,
}

impl GcDbRows {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for GcDbRows {
    async fn tick(&self) {
        let now = DateTimeAsMicroseconds::now();
        let data_to_expire = get_data_to_expire(self.app.as_ref(), now).await;

        if let Some(partitions_to_expire) = data_to_expire.partitions_to_expire {
            let now = DateTimeAsMicroseconds::now();
            for (db_table, partitions) in partitions_to_expire {
                crate::db_operations::write::delete_partitions(
                    self.app.as_ref(),
                    db_table.as_ref(),
                    partitions,
                    EventSource::as_gc(),
                    now,
                )
                .await;
            }
        }

        if let Some(db_rows_to_expire) = data_to_expire.db_rows_to_expire {
            let now = DateTimeAsMicroseconds::now();
            for (db_table, db_rows_to_expire) in db_rows_to_expire {
                crate::db_operations::write::bulk_delete(
                    self.app.as_ref(),
                    db_table.as_ref(),
                    db_rows_to_expire,
                    EventSource::as_gc(),
                    now,
                    now,
                )
                .await;
            }
        }
    }
}

async fn get_data_to_expire(app: &AppContext, now: DateTimeAsMicroseconds) -> DataToExpire {
    let mut tables_with_partitions_to_expire = LazyVec::new();

    let tables = app.db.get_tables().await;

    let mut rows_to_expire_by_table = LazyVec::new();

    for table in tables {
        let max_amount = table.attributes.get_max_partitions_amount();

        let table_read_access = table.data.read().await;

        if let Some(max_amount) = max_amount {
            if let Some(partitions_to_expire) =
                table_read_access.get_partitions_to_expire(max_amount)
            {
                tables_with_partitions_to_expire.push((table.clone(), partitions_to_expire));
            }
        }

        let mut db_rows_to_expire = LazyHashMap::new();
        for (partition_key, db_partition) in &table_read_access.partitions {
            if let Some(rows_to_expire) = db_partition.get_rows_to_expire(now) {
                db_rows_to_expire.insert(partition_key.to_string(), rows_to_expire);
            }
        }

        if let Some(db_rows) = db_rows_to_expire.get_result() {
            rows_to_expire_by_table.push((table.clone(), db_rows));
        }
    }

    DataToExpire {
        partitions_to_expire: tables_with_partitions_to_expire.get_result(),
        db_rows_to_expire: rows_to_expire_by_table.get_result(),
    }
}
