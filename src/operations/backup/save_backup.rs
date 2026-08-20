use my_logger::LogEventCtx;
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::app::AppContext;

use super::utils::*;

use std::sync::Arc;

use crate::app::DbNamespace;

/// Snapshots every namespace into a backup folder of its own. Returns the names
/// of the namespaces whose snapshot did NOT reach the disk — empty on a clean
/// run.
///
/// Every filesystem failure below is reported and swallowed, never unwrapped:
/// the whole loop runs inside a single timer tick, so one namespace panicking
/// took down the backup of every namespace after it in the list — and the
/// backup collector, which runs in the same tick. The names are returned rather
/// than only logged so that a caller which can answer a human — `MakeBackup` —
/// does not report success over a namespace that has no snapshot.
pub async fn save_backup(app: &AppContext, force_write: bool) -> Vec<String> {
    let mut failed = Vec::new();

    for db_namespace in app.namespaces.get_all() {
        if !save_namespace_backup(app, &db_namespace, force_write).await {
            failed.push(db_namespace.name.to_string());
        }
    }

    failed
}

/// Whether this namespace is left with its snapshots in the state they should
/// be in — which includes the namespaces there was nothing to do for.
async fn save_namespace_backup(
    app: &AppContext,
    db_namespace: &Arc<DbNamespace>,
    force_write: bool,
) -> bool {
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
                return true;
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
        return true;
    }

    let backup_content = super::super::build_db_snapshot_as_zip_archive(db_namespace).await;

    let file_name = now.to_rfc3339().replace(":", "").replace("-", "");

    let file_name = compile_backup_file(
        app,
        &db_namespace.name,
        format!("{}.zip", &file_name[..15]).as_str(),
    );

    if !write_backup_file(file_name.as_str(), backup_content).await {
        // The time stamp is deliberately left where it was: the next tick
        // retries instead of counting a snapshot which never reached the disk
        // as done and waiting out a whole interval.
        return false;
    }

    save_last_backup_time(app, db_namespace, now).await;

    true
}

/// Writes one file of a namespace's backup folder, creating the folder when it
/// is missing. Returns whether the content reached the disk.
///
/// The folder can be missing under a namespace that exists: `db/<ns>` is
/// created when the namespace is opened, `backup/<ns>` only when the namespace
/// is backed up for the first time. A namespace which never had a snapshot —
/// a namespace holding no tables, in particular — has the first but not the
/// second.
async fn write_backup_file(file_name: &str, content: Vec<u8>) -> bool {
    if let Some(folder) = std::path::Path::new(file_name).parent() {
        if let Err(err) = tokio::fs::create_dir_all(folder).await {
            my_logger::LOGGER.write_error(
                "Backup",
                format!("Can not create backup folder. Err: {}", err),
                LogEventCtx::new().add("folder", format!("{}", folder.display())),
            );
            return false;
        }
    }

    if let Err(err) = tokio::fs::write(file_name, content).await {
        my_logger::LOGGER.write_error(
            "Backup",
            format!("Can not write backup file. Err: {}", err),
            LogEventCtx::new().add("fileName", file_name.to_string()),
        );
        return false;
    }

    true
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

    write_backup_file(file_name.as_str(), now.to_rfc3339().into_bytes()).await;
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_missing_backup_folder_is_created_not_panicked_on() {
        // The production case: a namespace exists, its backup folder does not.
        // This used to be an unwrap() over ENOENT and it panicked the tick.
        let dir = std::env::temp_dir().join(format!("backup_write_{}", uuid::Uuid::new_v4()));
        let file_name = format!("{}/never-created-ns/.last_backup_time", dir.display());

        assert!(super::write_backup_file(file_name.as_str(), b"content".to_vec()).await);
        assert_eq!(
            b"content".to_vec(),
            tokio::fs::read(file_name.as_str()).await.unwrap()
        );

        tokio::fs::remove_dir_all(dir).await.ok();
    }

    #[tokio::test]
    async fn test_unwritable_path_is_reported_not_panicked_on() {
        // A file where a folder is expected: create_dir_all can not resolve it,
        // so the write is refused — and the caller keeps its time stamp.
        let dir = std::env::temp_dir().join(format!("backup_write_{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.unwrap();

        let occupied = format!("{}/ns", dir.display());
        tokio::fs::write(occupied.as_str(), b"i am a file")
            .await
            .unwrap();

        assert_eq!(
            false,
            super::write_backup_file(format!("{}/backup.zip", occupied).as_str(), b"x".to_vec())
                .await
        );

        tokio::fs::remove_dir_all(dir).await.ok();
    }
}
