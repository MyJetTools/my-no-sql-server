use crate::app::AppContext;
use my_http_server_swagger::*;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct NonInitializedModel {
    #[serde(rename = "initializingSeconds")]
    loading_time: i64,
}

impl NonInitializedModel {
    pub async fn new(app: &AppContext) -> Self {
        let now = DateTimeAsMicroseconds::now();

        Self {
            loading_time: now.seconds_before(app.created),
        }
    }
}
