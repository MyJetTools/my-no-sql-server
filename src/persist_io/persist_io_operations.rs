use crate::db::{db_snapshots::DbPartitionSnapshot, DbPartition, DbTableAttributesSnapshot};

pub enum TableLoadItem {
    TableAttributes(DbTableAttributesSnapshot),
    DbPartition {
        partition_key: String,
        db_partition: DbPartition,
    },
}

impl TableLoadItem {
    pub fn as_str(&self) -> &str {
        match self {
            TableLoadItem::TableAttributes(_) => "attr",
            TableLoadItem::DbPartition {
                partition_key,
                db_partition,
            } => partition_key,
        }
    }
}

#[async_trait::async_trait]
pub trait PersistIoOperations {
    async fn get_list_of_tables(&self) -> Vec<String>;

    async fn start_loading_table(&self, table_name: &str) -> Option<TableLoadItem>;

    async fn continue_loading_table(&self, table_name: &str) -> Option<TableLoadItem>;

    async fn create_table(&self, table_name: &str, attr: &DbTableAttributesSnapshot);

    async fn save_table_attributes(&self, table_name: &str, attr: &DbTableAttributesSnapshot);

    async fn save_partition(
        &self,
        table_name: &str,
        partition_key: &str,
        db_partition_snapshot: &DbPartitionSnapshot,
    );

    async fn delete_table(&self, table_name: &str);
    async fn delete_partition(&self, table_name: &str, partition_key: &str);
}
