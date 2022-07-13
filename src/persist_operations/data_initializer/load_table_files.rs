use std::{sync::Arc, time::Duration};

use crate::{app::AppContext, persist_io::TableFile};

use super::{load_tasks::NextFileToLoadResult, LoadedTableItem};

pub async fn spawn(app: Arc<AppContext>) {
    loop {
        let next_file_to_load = app.init_state.get_next_file_to_load().await;

        match next_file_to_load {
            NextFileToLoadResult::FileToLoad {
                table_name,
                file_name,
            } => {
                let table_file = TableFile::from_file_name(file_name.as_str());

                if let Err(err) = table_file {
                    app.logs.add_error(
                        Some(file_name.to_string()),
                        crate::app::logs::SystemProcess::Init,
                        "init_tables".to_string(),
                        format!("Error loading table file {}: {}", file_name, err),
                        None,
                    );
                    continue;
                }

                let table_file = table_file.unwrap();

                let content = app
                    .persist_io
                    .load_table_file(table_name.as_str(), &table_file)
                    .await;

                if let Some(content) = content.as_ref() {
                    match LoadedTableItem::new(&table_file, content) {
                        Ok(table_item) => {
                            app.init_state
                                .upload_table_file(table_name.as_str(), file_name, table_item)
                                .await;
                        }
                        Err(err) => {
                            app.logs.add_error(
                                Some(file_name.to_string()),
                                crate::app::logs::SystemProcess::Init,
                                "init_tables".to_string(),
                                format!("Error parsing table file {}: {}", file_name, err),
                                None,
                            );
                        }
                    }
                }
            }
            NextFileToLoadResult::NotReadyYet => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            NextFileToLoadResult::NothingToLoad => return,
        }
    }
}
