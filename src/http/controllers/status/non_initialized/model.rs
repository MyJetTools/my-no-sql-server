use crate::{app::AppContext, persist_operations::data_initializer::TableInitState};
use my_http_server_swagger::*;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct NonInitializedModel {
    #[serde(rename = "tablesRemains")]
    tables_remains: usize,
    #[serde(rename = "initializingSeconds")]
    initializing_seconds: i64,
    progress: Vec<TableProgressModel>,
}

impl NonInitializedModel {
    pub async fn new(app: &AppContext) -> Self {
        let (snapshot, tables_remains) = app.init_state.get_snapshot().await;

        let mut progress = Vec::new();

        let now = DateTimeAsMicroseconds::now();

        for (table_name, init_state) in snapshot {
            progress.push(TableProgressModel::new(table_name, init_state));
        }

        return Self {
            progress,
            tables_remains,
            initializing_seconds: now.seconds_before(app.created),
        };
    }
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct TableProgressModel {
    #[serde(rename = "tableName")]
    table_name: String,
    partitions: usize,
    loaded: usize,
    #[serde(rename = "secondsGone")]
    seconds_gone: i64,
}

impl TableProgressModel {
    pub fn new(table_name: String, init_state: TableInitState) -> Self {
        let now = DateTimeAsMicroseconds::now();
        Self {
            table_name,
            loaded: init_state.loaded,
            partitions: init_state.partitions_amount,
            seconds_gone: now.seconds_before(init_state.started),
        }
    }
}
