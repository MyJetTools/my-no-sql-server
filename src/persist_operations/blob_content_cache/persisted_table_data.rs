use my_no_sql_sdk::core::db::{
    db_table_master_node::PartitionLastWriteMoment, DbTable, DbTableAttributes,
};
use my_no_sql_sdk::core::rust_extensions::sorted_vec::{EntityWithStrKey, SortedVecWithStrKey};

pub struct PersistedTableData {
    pub table_name: String,
    pub attr: DbTableAttributes,
    pub partitions: SortedVecWithStrKey<PartitionLastWriteMoment>,
}

impl EntityWithStrKey for PersistedTableData {
    fn get_key(&self) -> &str {
        self.table_name.as_str()
    }
}

impl PersistedTableData {
    pub fn new(table_name: String, attr: DbTableAttributes) -> Self {
        Self {
            table_name,
            attr,
            partitions: SortedVecWithStrKey::new(),
        }
    }

    pub fn init(db_table: &DbTable) -> Self {
        Self {
            table_name: db_table.name.clone(),
            attr: db_table.attributes.clone(),
            partitions: db_table.get_partitions_last_write_moment(),
        }
    }
}
