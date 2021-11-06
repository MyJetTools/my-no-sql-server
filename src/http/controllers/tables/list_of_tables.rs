use crate::{app::AppContext, http::http_ok::HttpOkResult};
use std::result::Result;

use my_http_utils::HttpFailResult;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TableJsonResult {
    pub name: String,
    pub persist: bool,
    #[serde(rename = "maxPartitionsAmount")]
    pub max_partitions_amount: Option<usize>,
}

pub async fn get(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let tables = app.db.get_tables().await;

    let mut response: Vec<TableJsonResult> = vec![];

    for db_table in &tables {
        let attr = db_table.get_attributes().await;
        response.push(TableJsonResult {
            name: db_table.name.to_string(),
            persist: attr.persist,
            max_partitions_amount: attr.max_partitions_amount,
        });
    }

    return HttpOkResult::create_json_response(response);
}
