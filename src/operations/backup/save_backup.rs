use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::app::AppContext;

use super::utils::*;

pub async fn save_backup(app: &AppContext, force_write: bool) {
    let now = DateTimeAsMicroseconds::now();

    if !force_write {
        if let Some(last_backup_time) = get_last_backup_time(app).await {
            let backup_interval_seconds = app.settings.backup_interval_hours * 60 * 60;

            if now
                .duration_since(last_backup_time)
                .as_positive_or_zero()
                .as_secs()
                < backup_interval_seconds
            {
                return;
            }
        }
    }

    let backup_content = super::super::build_db_snapshot_as_zip_archive(app).await;

    let file_name = now.to_rfc3339().replace(":", "").replace("-", "");

    let file_name = compile_backup_file(app, format!("{}.zip", &file_name[..15]).as_str());

    tokio::fs::write(file_name.as_str(), backup_content)
        .await
        .unwrap();

    save_last_backup_time(app, now).await;
}

async fn get_last_backup_time(app: &AppContext) -> Option<DateTimeAsMicroseconds> {
    let file_name = compile_backup_file(app, LAST_TIME_BACKUP_FILE_NAME);

    let content = tokio::fs::read(file_name.as_str()).await;

    if content.is_err() {
        println!("Can not open file: {}", file_name.as_str());
        return None;
    }

    let content = content.unwrap();

    let content = String::from_utf8(content);

    if content.is_err() {
        println!("Can not parse file: {}", file_name.as_str());
        return None;
    }

    let content = content.unwrap();

    let result = DateTimeAsMicroseconds::from_str(content.as_str());

    if result.is_none() {
        println!("Can not parse date_time from file: {}", file_name.as_str());
    }

    result
}

async fn save_last_backup_time(app: &AppContext, now: DateTimeAsMicroseconds) {
    let file_name = compile_backup_file(app, LAST_TIME_BACKUP_FILE_NAME);

    let backup_content = now.to_rfc3339();
    tokio::fs::write(file_name.as_str(), backup_content)
        .await
        .unwrap();
}
