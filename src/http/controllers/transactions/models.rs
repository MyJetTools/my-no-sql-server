use my_http_server::types::RawDataTyped;
use my_http_server_swagger::*;
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct ProcessTransactionInputModel {
    #[http_query(name = "transactionId" description = "Id of transaction")]
    pub transaction_id: String,

    #[http_body_raw(description = "Process transaction")]
    pub body: RawDataTyped<JsonBaseTransaction>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct JsonBaseTransaction {
    #[serde(rename = "type")]
    pub transaction_type: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct StartTransactionResponse {
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
}
