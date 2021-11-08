use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{app::AppContext, db::DbRow, db_json_entity::JsonTimeStamp};

pub async fn start(app: Arc<AppContext>) {
    let duration = Duration::from_secs(1);
    while !app.states.is_initialized() {
        tokio::time::sleep(duration).await;
    }

    while !app.states.is_shutting_down() {
        for _ in 0..10 {
            if app.states.is_shutting_down() {
                break;
            }
        }

        let tick_result = tokio::spawn(interation(app.clone())).await;

        if let Err(err) = tick_result {
            app.logs
                .add_fatal_error(
                    crate::app::logs::SystemProcess::Timer,
                    "db_rows_expirator".to_string(),
                    format!("{}", err),
                )
                .await;
        }
    }
}

async fn interation(app: Arc<AppContext>) {
    let now = JsonTimeStamp::now();

    let removed = app.rows_with_expiration.remove_up_to(now.date_time).await;

    if removed.is_none() {
        return;
    }

    let removed = removed.unwrap();

    for (table_name, rows_to_delete) in removed {
        let db_table = app.db.get_table(table_name.as_str()).await;

        if let Some(db_table) = db_table {
            let attr = crate::operations::transaction_attributes::create(
                app.as_ref(),
                crate::db_sync::DataSynchronizationPeriod::Sec5,
            );

            crate::db_operations::write::bulk_delete::execute(
                app.as_ref(),
                db_table.as_ref(),
                as_hash_map(rows_to_delete),
                Some(attr),
                &now,
            )
            .await;
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
