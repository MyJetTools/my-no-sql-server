use crate::{
    app::AppContext, persist_operations::data_initializer::load_tasks::InitTableStateSnapshot,
};
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
    #[serde(rename = "tableBeingLoadedFiles")]
    table_being_loaded_files: Option<String>,
}

impl NonInitializedModel {
    pub async fn new(app: &AppContext) -> Self {
        let snapshot = app.init_state.get_snapshot().await;

        let mut progress = Vec::new();

        let now = DateTimeAsMicroseconds::now();

        for table_snapshot in snapshot.loading {
            progress.push(TableProgressModel::new(table_snapshot));
        }

        return Self {
            progress,
            tables_remains: snapshot.to_load.len(),
            initializing_seconds: now.seconds_before(app.created),
            table_being_loaded_files: snapshot.table_being_loaded_files,
        };
    }
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct TableProgressModel {
    #[serde(rename = "tableName")]
    table_name: String,
    #[serde(rename = "toLoad")]
    to_load: usize,
    loaded: usize,
    #[serde(rename = "secondsGone")]
    seconds_gone: i64,
}

impl TableProgressModel {
    pub fn new(table_snapshot: InitTableStateSnapshot) -> Self {
        let seconds_gone = if let Some(init_started) = table_snapshot.init_started {
            let now = DateTimeAsMicroseconds::now();
            now.seconds_before(init_started)
        } else {
            0
        };

        Self {
            table_name: table_snapshot.name,
            loaded: table_snapshot.loaded,
            to_load: table_snapshot.to_load,
            seconds_gone,
        }
    }
}
