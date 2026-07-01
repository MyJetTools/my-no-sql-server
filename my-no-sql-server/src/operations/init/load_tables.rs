use std::sync::Arc;

use my_no_sql_sdk::server::rust_extensions::StopWatch;

use crate::app::AppContext;

use super::partitions_init_reader::PartitionsInitReader;

pub async fn load_tables(app: Arc<AppContext>) {
    let sw = StopWatch::new();

    if let Some(server_url) = app.settings.get_init_from_other_server_url() {
        let tables = super::from_other_instance::load_tables(server_url).await;
        let tables_amount = tables.len();

        let entities_reader = super::from_other_instance::EntitiesInitReaderForOtherInstance::new(
            server_url.to_string(),
        );

        super::scripts::init_tables(&app, tables, entities_reader, true).await;

        println!("Tables loaded: {} in {:?}", tables_amount, sw.duration());
    } else {
        let tables = app.repo.get_tables().await;
        let tables_amount = tables.len();
        let partitions = app
            .repo
            .load_all_partitions(app.settings.skip_broken_partitions)
            .await;
        let entities_reader =
            PartitionsInitReader::new(partitions, app.settings.skip_broken_partitions);
        super::scripts::init_tables(&app, tables, entities_reader, false).await;

        println!("Tables loaded: {} in {:?}", tables_amount, sw.duration());
    }

    app.states.set_initialized();
}
