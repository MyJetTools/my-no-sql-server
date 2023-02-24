use my_no_sql_tcp_shared::{DeleteRowTcpContract, MyNoSqlTcpContract};

use crate::db_sync::SyncEvent;
use my_json::json_reader::consts::EMPTY_ARRAY;

pub async fn serialize(sync_event: &SyncEvent, compress: bool) -> Option<Vec<u8>> {
    match sync_event {
        SyncEvent::TableFirstInit(sync_data) => {
            let table_snapshot = sync_data.db_table.get_table_snapshot().await;

            let data = table_snapshot.as_json_array().build();

            let tcp_contract = MyNoSqlTcpContract::InitTable {
                table_name: sync_data.db_table.name.to_string(),
                data,
            };

            if compress {
                return tcp_contract.compress_if_make_sence_and_serialize().into();
            }

            return tcp_contract.serialize().into();
        }
        SyncEvent::UpdateTableAttributes(_) => None,
        SyncEvent::InitTable(sync_data) => {
            let data = sync_data.table_snapshot.as_json_array().build();

            let tcp_contract = MyNoSqlTcpContract::InitTable {
                table_name: sync_data.table_data.table_name.to_string(),
                data,
            };

            if compress {
                return tcp_contract.compress_if_make_sence_and_serialize().into();
            }

            return tcp_contract.serialize().into();
        }
        SyncEvent::InitPartitions(data) => {
            let mut result = Vec::new();

            for (partition_key, snapshot) in &data.partitions_to_update {
                let tcp_contract = MyNoSqlTcpContract::InitPartition {
                    partition_key: partition_key.to_string(),
                    table_name: data.table_data.table_name.to_string(),
                    data: if let Some(db_partition_snapshot) = snapshot {
                        db_partition_snapshot
                            .db_rows_snapshot
                            .as_json_array()
                            .build()
                    } else {
                        EMPTY_ARRAY.to_vec()
                    },
                };

                if compress {
                    let payload = tcp_contract.compress_if_make_sence_and_serialize();
                    result.extend_from_slice(&payload);
                } else {
                    tcp_contract.serialize_into(&mut result);
                }
            }

            result.into()
        }
        SyncEvent::UpdateRows(data) => {
            let tcp_contract = MyNoSqlTcpContract::UpdateRows {
                table_name: data.table_data.table_name.to_string(),
                data: data.rows_by_partition.as_json_array().build(),
            };

            if compress {
                return tcp_contract.compress_if_make_sence_and_serialize().into();
            }

            return tcp_contract.serialize().into();
        }
        SyncEvent::DeleteRows(data) => {
            let mut result = Vec::new();

            if let Some(deleted_partitions) = &data.deleted_partitions {
                for (partition_key, _) in deleted_partitions {
                    MyNoSqlTcpContract::InitPartition {
                        table_name: data.table_data.table_name.to_string(),
                        partition_key: partition_key.to_string(),
                        data: EMPTY_ARRAY.to_vec(),
                    }
                    .serialize_into(&mut result);
                }
            }

            if let Some(deleted_rows) = &data.deleted_rows {
                for (partition_key, rows) in deleted_rows {
                    let mut deleted_rows = Vec::new();

                    for row_key in rows.keys() {
                        let contract = DeleteRowTcpContract {
                            partition_key: partition_key.to_string(),
                            row_key: row_key.to_string(),
                        };

                        deleted_rows.push(contract);
                    }

                    let contract = MyNoSqlTcpContract::DeleteRows {
                        table_name: data.table_data.table_name.to_string(),
                        rows: deleted_rows,
                    };

                    contract.serialize_into(&mut result);
                }
            }

            result.into()
        }
        SyncEvent::DeleteTable(data) => {
            let contract = MyNoSqlTcpContract::InitTable {
                table_name: data.table_data.table_name.to_string(),
                data: EMPTY_ARRAY.to_vec(),
            };

            contract.serialize().into()
        }
    }
}
