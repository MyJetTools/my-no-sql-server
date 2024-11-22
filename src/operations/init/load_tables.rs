use std::sync::Arc;

use my_no_sql_server_core::rust_extensions::StopWatch;

use crate::app::AppContext;

use super::entities_from_sqlite_reader::EntitiesFromSqliteReader;

pub async fn load_tables(app: Arc<AppContext>) {
    let mut sw = StopWatch::new();
    sw.start();

    if let Some(server_url) = app.settings.get_init_from_other_server_url() {
        let tables = super::from_other_instance::load_tables(server_url).await;
        let tables_amount = tables.len();

        let entities_reader = super::from_other_instance::EntitiesInitReaderForOtherInstance::new(
            server_url.to_string(),
        );

        super::scripts::init_tables(&app, tables, entities_reader, true).await;

        println!(
            "Tables loaded: {} in {:?} seconds",
            tables_amount,
            sw.duration()
        );
    } else {
        let tables = app.repo.get_tables().await;
        let tables_amount = tables.len();
        let entities = app.repo.get_all_entities().await;
        let entities_reader =
            EntitiesFromSqliteReader::new(entities, app.settings.skip_broken_partitions);
        super::scripts::init_tables(&app, tables, entities_reader, false).await;
        sw.pause();
        println!(
            "Tables loaded: {} in {:?} seconds",
            tables_amount,
            sw.duration()
        );
    }

    app.states.set_initialized();
}
