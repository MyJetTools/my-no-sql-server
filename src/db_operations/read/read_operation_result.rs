use std::{collections::HashMap, sync::Arc};

use my_json::json_writer::JsonArrayWriter;
use my_no_sql_sdk::core::db::DbRow;
use my_no_sql_server_core::DbTableWrapper;

use crate::{app::AppContext, db_operations::UpdateStatistics};

pub enum ReadOperationResult {
    SingleRow(Vec<u8>),
    RowsArray(Vec<u8>),
    EmptyArray,
}

impl ReadOperationResult {
    pub async fn compile_array_or_empty(
        app: &Arc<AppContext>,
        db_table: &Arc<DbTableWrapper>,
        db_rows: Option<HashMap<String, Vec<&Arc<DbRow>>>>,
        update_statistics: UpdateStatistics,
    ) -> Self {
        if db_rows.is_none() {
            return Self::EmptyArray;
        }

        let mut json_array_writer = JsonArrayWriter::new();

        let db_rows = db_rows.unwrap();

        if update_statistics.has_statistics_to_update() {
            for (partition_key, db_rows) in &db_rows {
                update_statistics
                    .update_statistics(app, db_table, partition_key, || {
                        db_rows.iter().map(|db_row| db_row.get_row_key())
                    })
                    .await;
            }
        }

        for (_, db_rows) in db_rows {
            for db_row in db_rows {
                json_array_writer.write(db_row.as_ref());
            }
        }

        return ReadOperationResult::RowsArray(json_array_writer.build());
    }

    pub async fn compile_array_or_empty_from_partition(
        app: &Arc<AppContext>,
        db_table: &Arc<DbTableWrapper>,
        partition_key: &String,
        db_rows: Option<Vec<&Arc<DbRow>>>,
        update_statistics: UpdateStatistics,
    ) -> Self {
        if db_rows.is_none() {
            return Self::EmptyArray;
        }

        let mut json_array_writer = JsonArrayWriter::new();

        let db_rows = db_rows.unwrap();

        update_statistics
            .update_statistics(app, db_table, partition_key, || {
                db_rows.iter().map(|db_row| db_row.get_row_key())
            })
            .await;

        for db_row in db_rows {
            json_array_writer.write(db_row.as_ref());
        }

        return ReadOperationResult::RowsArray(json_array_writer.build());
    }
}
