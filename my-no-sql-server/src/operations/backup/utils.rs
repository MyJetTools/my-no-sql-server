use std::path::MAIN_SEPARATOR;

use crate::app::AppContext;

pub const LAST_TIME_BACKUP_FILE_NAME: &str = ".last_backup_time";

pub fn compile_backup_file<'s>(app: &AppContext, file_name: &str) -> String {
    let backup_folder = app.settings.get_backup_folder();
    if backup_folder.as_str().ends_with(MAIN_SEPARATOR) {
        format!("{}{}", backup_folder.as_str(), file_name)
    } else {
        format!("{}{}{}", backup_folder.as_str(), MAIN_SEPARATOR, file_name)
    }
}
