use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use my_no_sql_sdk::core::db::DbTableAttributes;
use my_no_sql_sdk::server::rust_extensions::StopWatch;

use crate::app::AppContext;
use crate::persist_repo::LoadedTableAttrs;

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
        let mut tables = app.repo.get_tables().await;
        let partitions = app
            .repo
            .load_all_partitions(app.settings.skip_broken_partitions)
            .await;

        // Data-safety net: a partition can only exist without a metadata record
        // if the metadata was lost (corruption / external edit) — table creation
        // persists attributes before partitions, and `delete_table` removes
        // partitions before metadata. Recreate such orphan tables with default
        // attributes so their rows are loaded, not silently dropped.
        let known: HashSet<&str> = tables.iter().map(|t| t.table_name.as_str()).collect();
        let mut orphans: BTreeMap<String, usize> = BTreeMap::new();
        for partition in &partitions {
            if !known.contains(partition.table_name.as_str()) {
                *orphans.entry(partition.table_name.clone()).or_default() += 1;
            }
        }
        drop(known);
        for (table_name, partitions_amount) in orphans {
            println!(
                "WARNING: Table '{}' has {} persisted partition(s) but no metadata record. Recreating it with default attributes so its data is not lost.",
                table_name, partitions_amount
            );
            tables.push(LoadedTableAttrs {
                table_name: table_name.into(),
                attr: DbTableAttributes::create_default(),
            });
        }

        let tables_amount = tables.len();
        let entities_reader =
            PartitionsInitReader::new(partitions, app.settings.skip_broken_partitions);
        super::scripts::init_tables(&app, tables, entities_reader, false).await;

        println!("Tables loaded: {} in {:?}", tables_amount, sw.duration());
    }

    app.states.set_initialized();
}
