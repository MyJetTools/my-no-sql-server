use std::sync::Arc;

use crate::{
    app::AppContext,
    db::{DbTableAttributesSnapshot, DbTableData},
    persist_operations::data_initializer::load_tasks::TableLoadingTask,
};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::LoadedTableItem;

pub async fn load_table(
    app: &Arc<AppContext>,
    table_loading_task: &Arc<TableLoadingTask>,
    file_name: String,
) {
    app.logs.add_info(
        Some(table_to_load.table_name.to_string()),
        crate::app::logs::SystemProcess::Init,
        "init_tables".to_string(),
        format!("Initializing table {}", table_to_load.table_name),
    );

    let now = DateTimeAsMicroseconds::now();

    let mut db_table_data = DbTableData::new(table_to_load.table_name.to_string(), now);

    let mut db_table_attirbutes: Option<DbTableAttributesSnapshot> = None;

    let table_items = super::load_table_files(app.clone(), table_to_load).await;

    for table_item in table_items.get().await {
        match table_item {
            LoadedTableItem::TableAttributes(attr) => {
                db_table_attirbutes = Some(attr);
            }
            LoadedTableItem::DbPartition {
                partition_key,
                db_partition,
            } => {
                db_table_data.partitions.insert(partition_key, db_partition);
            }
        }
    }

    let attr = if let Some(attr) = db_table_attirbutes {
        attr
    } else {
        DbTableAttributesSnapshot::create_default()
    };

    crate::db_operations::write::table::init(app.as_ref(), db_table_data, attr).await;
}
