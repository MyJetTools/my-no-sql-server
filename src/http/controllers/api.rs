use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::db::{FailOperationResult, OperationResult};

pub fn is_alive() -> Result<OperationResult, FailOperationResult> {
    let version = env!("CARGO_PKG_VERSION");

    let env_info = env!("ENV_INFO");

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let model = ApiModel {
        name: "MyNoSqlServer.Api".to_string(),
        time: time,
        version: version.to_string(),
        env_info: env_info.to_string(),
    };

    return OperationResult::create_json_response(model);
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiModel {
    name: String,
    time: u64,
    version: String,
    env_info: String,
}
