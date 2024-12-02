use crate::app::AppContext;

#[derive(Debug)]
pub enum BackupError {
    TableNotFoundInBackupFile,
}

pub async fn restore(
    app: &AppContext,
    backup_content: Vec<u8>,
    table_name: Option<&str>,
) -> Result<(), BackupError> {
    let zip_cursor = std::io::Cursor::new(backup_content);

    let zip = zip::ZipArchive::new(zip_cursor).unwrap();

    for itm in zip.file_names() {
        println!("File: {}", itm);
    }

    Ok(())
}
