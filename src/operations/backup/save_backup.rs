use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::app::AppContext;

use super::utils::*;

use std::sync::Arc;

use crate::app::DbNamespace;

/// Snapshots every namespace into a backup folder of its own.
pub async fn save_backup(app: &AppContext, force_write: bool) {
    for db_namespace in app.namespaces.get_all() {
        save_namespace_backup(app, &db_namespace, force_write).await;
    }
}

async fn save_namespace_backup(
    app: &AppContext,
    db_namespace: &Arc<DbNamespace>,
    force_write: bool,
) {
    let now = DateTimeAsMicroseconds::now();

    if !force_write {
        if let Some(last_backup_time) = get_last_backup_time(app, db_namespace).await {
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

    // A namespace with no tables produces an empty archive — 22 bytes, just the
    // zip end-of-central-directory record. Written every interval it is
    // indistinguishable from a real snapshot to the collector, so it occupies a
    // MaxBackupsToKeep slot forever and pushes a genuine backup out. Nothing to
    // back up means no file at all.
    //
    // The timestamp is still moved forward, so an empty namespace is
    // re-checked once per interval rather than on every single tick.
    if db_namespace.tables_amount() == 0 {
        println!(
            "Namespace '{}' holds no tables — skipping the backup",
            db_namespace.name
        );
        save_last_backup_time(app, db_namespace, now).await;
        return;
    }

    let backup_content = super::super::build_db_snapshot_as_zip_archive(db_namespace).await;

    let file_name = now.to_rfc3339().replace(":", "").replace("-", "");

    let file_name = compile_backup_file(
        app,
        &db_namespace.name,
        format!("{}.zip", &file_name[..15]).as_str(),
    );

    if let Some(folder) = std::path::Path::new(file_name.as_str()).parent() {
        let _ = tokio::fs::create_dir_all(folder).await;
    }

    tokio::fs::write(file_name.as_str(), backup_content)
        .await
        .unwrap();

    save_last_backup_time(app, db_namespace, now).await;
}

async fn get_last_backup_time(
    app: &AppContext,
    db_namespace: &Arc<DbNamespace>,
) -> Option<DateTimeAsMicroseconds> {
    let file_name = compile_backup_file(app, &db_namespace.name, LAST_TIME_BACKUP_FILE_NAME);

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

async fn save_last_backup_time(
    app: &AppContext,
    db_namespace: &Arc<DbNamespace>,
    now: DateTimeAsMicroseconds,
) {
    let file_name = compile_backup_file(app, &db_namespace.name, LAST_TIME_BACKUP_FILE_NAME);

    let backup_content = now.to_rfc3339();
    tokio::fs::write(file_name.as_str(), backup_content)
        .await
        .unwrap();
}
