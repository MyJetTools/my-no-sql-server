use std::collections::{BTreeMap, HashMap};

use my_http_server::{HttpOkResult, HttpOutput, WebContentType};
use my_json::json_writer::{JsonArrayWriter, JsonObjectWriter};

use crate::{
    db::db_snapshots::DbPartitionSnapshot,
    db_sync::{
        states::{DeleteRowsEventSyncData, UpdateRowsSyncData},
        SyncEvent,
    },
};

pub async fn convert(sync_event: &SyncEvent) -> Option<HttpOkResult> {
    match sync_event {
        SyncEvent::TableFirstInit(sync_data) => {
            let content = sync_data.db_table.get_table_as_json_array().await;
            compile_init_table_result(sync_data.db_table.name.as_str(), content).into()
        }
        SyncEvent::InitTable(sync_data) => {
            let content = sync_data.table_snapshot.as_json_array();
            compile_init_table_result(sync_data.table_data.table_name.as_str(), content).into()
        }
        SyncEvent::InitPartitions(sync_data) => {
            compile_init_partitions_result(&sync_data.partitions_to_update).into()
        }
        SyncEvent::UpdateRows(sync_data) => compile_update_rows_result(sync_data).into(),
        SyncEvent::DeleteRows(sync_data) => compile_delete_rows_result(sync_data).into(),
        SyncEvent::DeleteTable(sync_data) => compile_init_table_result(
            sync_data.table_data.table_name.as_str(),
            JsonArrayWriter::new(),
        )
        .into(),
        SyncEvent::UpdateTableAttributes(_) => None,
    }
}

const SYNC_HEADER: &str = "sync";

fn compile_init_table_result(table_name: &str, content: JsonArrayWriter) -> HttpOkResult {
    let mut headers = HashMap::new();
    headers.insert("SyncType".to_string(), format!("initTable={table_name}"));

    let output = HttpOutput::Content {
        headers: Some(headers),
        content_type: Some(WebContentType::Json),
        content: content.build(),
    };

    HttpOkResult {
        write_telemetry: false,
        output,
    }
}

fn compile_init_partitions_result(
    partitions: &BTreeMap<String, Option<DbPartitionSnapshot>>,
) -> HttpOkResult {
    let mut json_object_writer = JsonObjectWriter::new();

    for (partition_key, db_partition) in partitions {
        if let Some(db_partition_snapshot) = db_partition {
            json_object_writer
                .write_object(partition_key, db_partition_snapshot.db_rows.as_json_array());
        } else {
            json_object_writer.write_empty_array(partition_key)
        }
    }

    let mut headers = HashMap::new();
    headers.insert(SYNC_HEADER.to_string(), "initPartition".to_string());

    let output = HttpOutput::Content {
        headers: Some(headers),
        content_type: Some(WebContentType::Json),
        content: json_object_writer.build(),
    };

    HttpOkResult {
        write_telemetry: false,
        output,
    }
}

pub fn compile_update_rows_result(sync_data: &UpdateRowsSyncData) -> HttpOkResult {
    let mut headers = HashMap::new();
    headers.insert(
        SYNC_HEADER.to_string(),
        format!("updateRows={}", sync_data.table_data.table_name),
    );

    let output = HttpOutput::Content {
        headers: Some(headers),
        content_type: Some(WebContentType::Json),
        content: sync_data.rows_by_partition.as_json_array().build(),
    };

    HttpOkResult {
        write_telemetry: false,
        output,
    }
}

pub fn compile_delete_rows_result(sync_data: &DeleteRowsEventSyncData) -> HttpOkResult {
    let mut headers = HashMap::new();
    headers.insert(
        SYNC_HEADER.to_string(),
        format!("deleteRows={}", sync_data.table_data.table_name),
    );

    let output = HttpOutput::Content {
        headers: Some(headers),
        content_type: Some(WebContentType::Json),
        content: sync_data.as_vec(),
    };

    HttpOkResult {
        write_telemetry: false,
        output,
    }
}

pub fn compile_ping_result() -> HttpOkResult {
    let mut headers = HashMap::new();
    headers.insert(SYNC_HEADER.to_string(), format!("ping"));

    let output = HttpOutput::Content {
        headers: Some(headers),
        content_type: Some(WebContentType::Json),
        content: vec![],
    };

    HttpOkResult {
        write_telemetry: false,
        output,
    }
}
