use std::{sync::Arc, time::SystemTime};

use serde::{Deserialize, Serialize};

use crate::{
    app::AppServices,
    db::{FailOperationResult, OperationResult},
    utils::{StopWatch, StringBuilder},
};

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

    let json = serde_json::to_string(&model).unwrap();

    return Ok(OperationResult::OkWithJsonString { json });
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiModel {
    name: String,
    time: u64,
    version: String,
    env_info: String,
}

pub async fn get_logs(app: Arc<AppServices>) -> Result<OperationResult, FailOperationResult> {
    let mut sw = StopWatch::new();
    sw.start();
    let logs = app.logs.get().await;

    let mut sb = StringBuilder::new();

    for log_item in logs {
        let line = format!(
            "{} {:?}",
            crate::utils::date_time::to_iso_string(log_item.date),
            log_item.level
        );
        sb.append_line(&line);

        let line = format!("Process: {}", log_item.process);
        sb.append_line(line.as_str());

        sb.append_line(log_item.message.as_str());

        if let Some(err_ctx) = log_item.err_ctx {
            let line = format!("ErrCTX: {}", err_ctx);
            sb.append_line(line.as_str());
        }

        sb.append_line("-----------------------------");
    }

    sw.pause();

    let line = format!("Rendered in {:?}", sw.duration());
    sb.append_line(line.as_str());

    Ok(OperationResult::Text {
        text: sb.to_string_utf8().unwrap(),
    })
}
