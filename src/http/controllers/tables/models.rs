use my_http_macros::MyHttpInput;
use my_http_server::middlewares::controllers::documentation::data_types::{
    HttpDataType, HttpField, HttpObjectStructure,
};
use serde::{Deserialize, Serialize};

use crate::db_sync::DataSynchronizationPeriod;

#[derive(Deserialize, Serialize)]
pub struct TableJsonResult {
    pub name: String,
    pub persist: bool,
    #[serde(rename = "maxPartitionsAmount")]
    pub max_partitions_amount: Option<usize>,
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
