use crate::{app::AppContext, zip::DbZipBuilder};

pub async fn build_db_snapshot_as_zip_archive(app: &AppContext) -> Vec<u8> {
    let tables = app.db.get_tables();

    let mut zip_builder = DbZipBuilder::new();

    for db_table in tables.iter() {
        let table_snapshot = db_table.get_table_snapshot();

        zip_builder
            .add_table(&db_table.name.as_str(), &table_snapshot)
            .unwrap();
    }

    zip_builder.get_payload().unwrap()
}
