use crate::{
    app::AppContext,
    http::{controllers::consts, http_ctx::HttpContext, http_ok::HttpOkResult},
};
use my_http_utils::HttpFailResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NewMultipartResponse {
    #[serde(rename = "snapshotId")]
    pub snapshot_id: String,
}

pub async fn get(app: &AppContext, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let result = crate::db_operations::read::multipart::start_read_all(app, table_name).await?;

    let response = NewMultipartResponse {
        snapshot_id: format!("{}", result),
    };

    return HttpOkResult::create_json_response(response);
}
