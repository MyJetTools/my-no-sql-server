use crate::app::AppContext;

use super::utils::compile_backup_file;

pub async fn gc_backups(app: &AppContext) {
    let mut result = super::get_list_of_files(app).await;

    while result.len() > app.settings.max_backups_to_keep {
        let file_name = result.pop().unwrap();
        println!("Deleting backup file: {}", file_name.as_str());
        delete_backup(app, file_name.as_str()).await;
    }
}

async fn delete_backup(app: &AppContext, file_name: &str) {
    let file_full_path = compile_backup_file(app, file_name);
    tokio::fs::remove_file(file_full_path).await.unwrap();
}
