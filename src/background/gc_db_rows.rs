use std::{collections::HashMap, sync::Arc};

use rust_extensions::{date_time::DateTimeAsMicroseconds, lazy::LazyVec, MyTimerTick};

use crate::{
    app::{logs::SystemProcess, AppContext},
    db::DbTableWrapper,
    db_sync::EventSource,
    utils::LazyHashMap,
};

struct DataToExpire {
    partitions_to_expire: Option<Vec<(Arc<DbTableWrapper>, Vec<String>)>>,
    db_rows_to_expire: Option<Vec<(Arc<DbTableWrapper>, HashMap<String, Vec<String>>)>>,
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
                let result = crate::db_operations::write::delete_partitions(
                    self.app.as_ref(),
                    db_table.as_ref(),
                    partitions,
                    EventSource::as_gc(),
                    now,
                )
                .await;

                if let Err(err) = result {
                    if !err.is_app_is_not_initialized() {
                        self.app.logs.add_error(
                            Some(db_table.name.to_string()),
                            SystemProcess::Timer,
                            "GcDbRows_timerTick".to_string(),
                            "Error Executon operation Delete Partitions".to_string(),
                            Some(format!("{:?}", err)),
                        )
                    }
                }
            }
        }

        if let Some(db_rows_to_expire) = data_to_expire.db_rows_to_expire {
            let now = DateTimeAsMicroseconds::now();
            for (db_table, db_rows_to_expire) in db_rows_to_expire {
                let result = crate::db_operations::write::bulk_delete(
                    self.app.as_ref(),
                    db_table.as_ref(),
                    db_rows_to_expire,
                    EventSource::as_gc(),
                    now,
                    now,
                )
                .await;

                if let Err(err) = result {
                    if !err.is_app_is_not_initialized() {
                        self.app.logs.add_error(
                            Some(db_table.name.to_string()),
                            SystemProcess::Timer,
                            "GcDbRows_timerTick".to_string(),
                            "Error Executon operation BulkDelete".to_string(),
                            Some(format!("{:?}", err)),
                        )
                    }
                }
            }
        }
    }
}

async fn get_data_to_expire(app: &AppContext, now: DateTimeAsMicroseconds) -> DataToExpire {
    let mut tables_with_partitions_to_expire = LazyVec::new();

    let tables = app.db.get_tables().await;

    let mut rows_to_expire_by_table = LazyVec::new();

    for table in tables {
        let read_access = table.data.read().await;

        if let Some(max_amount) = read_access.db_table.attributes.max_partitions_amount {
            if let Some(partitions_to_expire) =
                read_access.db_table.get_partitions_to_expire(max_amount)
            {
                tables_with_partitions_to_expire.add((table.clone(), partitions_to_expire));
            }
        }

        let mut db_rows_to_expire = LazyHashMap::new();

        for (partition_key, db_partition) in &read_access.db_table.partitions {
            if db_partition.get_rows_amount() == 0 {
                tables_with_partitions_to_expire
                    .add((table.clone(), vec![partition_key.to_string()]));
            }
            if let Some(rows_to_expire) = db_partition.get_rows_to_expire(now) {
                db_rows_to_expire.insert(partition_key.to_string(), rows_to_expire);
            }
        }

        if let Some(db_rows) = db_rows_to_expire.get_result() {
            rows_to_expire_by_table.add((table.clone(), db_rows));
        }
    }

    DataToExpire {
        partitions_to_expire: tables_with_partitions_to_expire.get_result(),
        db_rows_to_expire: rows_to_expire_by_table.get_result(),
    }
}
