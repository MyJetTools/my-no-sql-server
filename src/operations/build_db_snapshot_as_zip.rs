use std::sync::Arc;

use crate::{app::DbNamespace, zip::DbZipBuilder};

/// Zips a snapshot of every table of the namespace.
///
/// The archive is built in memory, so a failure here is not io and is not
/// expected to ever happen — but it is handed to the caller instead of
/// unwrapped: this is called from inside the backup tick, which goes on to back
/// up every other namespace after this one, and a panic there is precisely the
/// failure the rest of the module is written to avoid.
pub async fn build_db_snapshot_as_zip_archive(
    db_namespace: &Arc<DbNamespace>,
) -> Result<Vec<u8>, String> {
    let tables = db_namespace.db.get_tables();

    let mut zip_builder = DbZipBuilder::new();

    for db_table in tables.iter() {
        let table_snapshot = db_table.get_table_snapshot();

        if let Err(err) = zip_builder.add_table(db_table.name.as_str(), &table_snapshot) {
            return Err(format!(
                "Can not add the table {} to the archive. Err: {}",
                db_table.name, err
            ));
        }
    }

    zip_builder
        .get_payload()
        .map_err(|err| format!("Can not compile the archive. Err: {}", err))
}
