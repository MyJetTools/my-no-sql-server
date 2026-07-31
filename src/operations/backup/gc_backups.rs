use std::sync::Arc;

use crate::app::{AppContext, DbNamespace};

use super::utils::compile_backup_file;

/// `MaxBackupsToKeep` is counted inside each namespace: they back up
/// independently, so one busy namespace must not evict another one snapshots.
pub async fn gc_backups(app: &AppContext) {
    for db_namespace in app.namespaces.get_all() {
        let mut result = super::get_list_of_files(app, &db_namespace).await;

        while result.len() > app.settings.max_backups_to_keep {
            let file_name = result.pop().unwrap().name;
            println!(
                "Deleting backup file of the namespace {}: {}",
                db_namespace.name,
                file_name.as_str()
            );
            delete_backup(app, &db_namespace, file_name.as_str()).await;
        }
    }
}

async fn delete_backup(app: &AppContext, db_namespace: &Arc<DbNamespace>, file_name: &str) {
    let file_full_path = compile_backup_file(app, &db_namespace.name, file_name);
    tokio::fs::remove_file(file_full_path).await.unwrap();
}
