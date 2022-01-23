use std::str::FromStr;

use my_http_macros::{MyHttpInput, MyHttpObjectStructure};
use serde::{Deserialize, Serialize};

use crate::{db::DbTable, db_sync::DataSynchronizationPeriod};

#[derive(Deserialize, Serialize, MyHttpObjectStructure)]
pub struct TableContract {
    pub name: String,
    pub persist: bool,
    #[serde(rename = "maxPartitionsAmount")]
    pub max_partitions_amount: Option<usize>,
}

impl Into<TableContract> for &DbTable {
    fn into(self) -> TableContract {
        let table_snapshot = self.attributes.get_snapshot();
        TableContract {
            name: self.name.to_string(),
            persist: table_snapshot.persist,
            max_partitions_amount: table_snapshot.max_partitions_amount,
        }
    }
}

#[derive(MyHttpInput)]
pub struct CreateTableCotnract {
    #[http_query(name = "table"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(description = "Persist a table"; default="true")]
    pub persist: bool,

    #[http_query(name = "maxPartitionsAmount"; description = "Maximim partitions amount. Empty - means unlimited")]
    pub max_partitions_amount: Option<usize>,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default="Sec5")]
    pub sync_period: DataSynchronizationPeriod,
}

/*
pub fn table_name_doc() -> HttpDataType {
    let object_structure = HttpObjectStructure {
        struct_id: "TableResponse".to_string(),
        fields: vec![
            HttpField::new("name", HttpDataType::as_string(), true, None),
            HttpField::new("persist", HttpDataType::as_bool(), true, None),
            HttpField::new("maxPartitionsAmount", HttpDataType::as_long(), false, None),
        ],
    };
    HttpDataType::Object(object_structure)
}
 */
