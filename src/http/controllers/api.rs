use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::http::{http_fail::HttpFailResult, http_ok::HttpOkResult};

pub fn is_alive() -> Result<HttpOkResult, HttpFailResult> {
    let version = env!("CARGO_PKG_VERSION");

    let env_info = std::env::var("ENV_INFO");

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let model = ApiModel {
        name: "MyNoSqlServer.Api".to_string(),
        time: time,
        version: version.to_string(),
        env_info: if let Ok(value) = env_info {
            value
        } else {
            "".to_string()
        },
    };

    return HttpOkResult::create_json_response(model);
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiModel {
    name: String,
    time: u64,
    version: String,
    env_info: String,
}
