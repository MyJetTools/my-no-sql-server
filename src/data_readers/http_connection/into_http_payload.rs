use my_json::json_writer::JsonArrayWriter;

use crate::db_sync::{
    states::{DeleteRowsEventSyncData, InitPartitionsSyncData, UpdateRowsSyncData},
    SyncEvent,
};

pub async fn convert(sync_event: &SyncEvent) -> Option<Vec<u8>> {
    match sync_event {
        SyncEvent::TableFirstInit(sync_data) => {
            let content = sync_data.db_table.get_table_as_json_array().await;
            write_init_table_result(sync_data.db_table.name.as_str(), content).into()
        }
        SyncEvent::InitTable(sync_data) => {
            let content = sync_data.table_snapshot.as_json_array();
            write_init_table_result(sync_data.table_data.table_name.as_str(), content).into()
        }
        SyncEvent::InitPartitions(sync_data) => write_init_partitions_result(sync_data).into(),
        SyncEvent::UpdateRows(sync_data) => compile_update_rows_result(sync_data).into(),
        SyncEvent::DeleteRows(sync_data) => compile_delete_rows_result(sync_data).into(),
        SyncEvent::DeleteTable(sync_data) => write_init_table_result(
            sync_data.table_data.table_name.as_str(),
            JsonArrayWriter::new(),
        )
        .into(),
        SyncEvent::UpdateTableAttributes(_) => None,
    }
}

fn write_init_table_result(table_name: &str, content: JsonArrayWriter) -> Vec<u8> {
    let mut result = Vec::new();
    let command = format!("initTable={table_name}");
    write_pascal_string(command.as_str(), &mut result);

    let content = content.build();
    write_byte_array(content.as_slice(), &mut result);
    result
}

fn write_init_partitions_result(sync_data: &InitPartitionsSyncData) -> Vec<u8> {
    let mut result = Vec::new();
    let command = format!("initPartitions={}", sync_data.table_data.table_name);
    write_pascal_string(command.as_str(), &mut result);

    let content = sync_data.as_json().build();
    write_byte_array(content.as_slice(), &mut result);
    result
}

pub fn compile_update_rows_result(sync_data: &UpdateRowsSyncData) -> Vec<u8> {
    let mut result = Vec::new();
    let command = format!("updateRows={}", sync_data.table_data.table_name);
    write_pascal_string(command.as_str(), &mut result);

    let content = sync_data.rows_by_partition.as_json_array().build();
    write_byte_array(content.as_slice(), &mut result);
    result
}

pub fn compile_delete_rows_result(sync_data: &DeleteRowsEventSyncData) -> Vec<u8> {
    let mut result = Vec::new();
    let command = format!("deleteRows={}", sync_data.table_data.table_name);
    write_pascal_string(command.as_str(), &mut result);

    let content = sync_data.as_vec();
    write_byte_array(content.as_slice(), &mut result);
    result
}

fn write_pascal_string(src: &str, dest: &mut Vec<u8>) {
    let bytes = src.as_bytes();
    dest.push(bytes.len() as u8);
    dest.extend_from_slice(bytes)
}

fn write_byte_array(src: &[u8], dest: &mut Vec<u8>) {
    let bytes = src.len() as u32;

    dest.extend_from_slice(&bytes.to_le_bytes());
    dest.extend_from_slice(src);
}
