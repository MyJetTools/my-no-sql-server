use my_http_server_swagger::{MyHttpInput, MyHttpObjectStructure};
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct DataReaderGreetingInputModel {
    #[http_query(name = "name"; description = "Name of Application")]
    pub name: String,
    #[http_query(name = "version"; description = "Version of client library")]
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct DataReaderGreetingResult {
    #[serde(rename = "session")]
    pub session_id: String,
}

#[derive(MyHttpInput)]
pub struct SubscribeToTableInputModel {
    #[http_header(name = "session"; description = "Id of session")]
    pub session_id: String,

    #[http_query(name = "tableName"; description = "Table to subscriber")]
    pub table_name: String,
}

#[derive(MyHttpInput)]
pub struct GetChangesInputModel {
    #[http_header(name = "session"; description = "Id of session")]
    pub session_id: String,
}
