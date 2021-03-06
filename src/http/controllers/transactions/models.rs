use my_http_server_controllers::controllers::documentation::{
    data_types::HttpDataType, out_results::HttpResult,
};
use my_http_server_swagger::*;
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct ProcessTransactionInputModel {
    #[http_query(name = "transactionId" description = "Id of transaction")]
    pub transaction_id: String,

    #[http_body(description = "Process transaction" body_type="JsonBaseTransaction")]
    pub body: Vec<u8>,
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

pub fn transaction_not_found_response_doc() -> HttpResult {
    HttpResult {
        http_code: 401,
        nullable: false,
        description: "Transaction not found".to_string(),
        data_type: HttpDataType::None,
    }
}
