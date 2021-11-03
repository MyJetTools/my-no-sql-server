use my_no_sql_tcp_shared::{DeleteRowTcpContract, TcpContract};

use crate::{db::read_as_json::DbEntityAsJsonArray, db_sync::SyncEvent, json::consts::EMPTY_ARRAY};

pub enum TcpContractsToSend {
    None,
    Single(TcpContract),
    Multiple(Vec<TcpContract>),
}

pub fn into_tcp_contract(event: &SyncEvent) -> TcpContractsToSend {
    match event {
        SyncEvent::UpdateTableAttributes(_) => TcpContractsToSend::None,
        SyncEvent::InitTable(data) => {
            let result = TcpContract::InitTable {
                table_name: data.table_data.table_name.to_string(),
                data: data.as_raw_bytes(),
            };

            TcpContractsToSend::Single(result)
        }
        SyncEvent::InitPartitions(data) => {
            let mut result = Vec::new();

            for (partition_key, snapshot) in &data.partitions_to_update {
                let contract = TcpContract::InitPartition {
                    partition_key: partition_key.to_string(),
                    table_name: data.table_data.table_name.to_string(),
                    data: if let Some(snapshot) = snapshot {
                        snapshot.content.as_json_array()
                    } else {
                        EMPTY_ARRAY.to_vec()
                    },
                };

                result.push(contract);
            }

            TcpContractsToSend::Multiple(result)
        }
        SyncEvent::UpdateRows(data) => {
            let result = TcpContract::UpdateRows {
                table_name: data.table_data.table_name.to_string(),
                data: data.updated_rows_by_partition.as_json_array(),
            };
            TcpContractsToSend::Single(result)
        }
        SyncEvent::Delete(data) => {
            let mut result = Vec::new();

            if let Some(deleted_partitions) = &data.deleted_partitions {
                for (partition_key, _) in deleted_partitions {
                    result.push(TcpContract::InitPartition {
                        table_name: data.table_data.table_name.to_string(),
                        partition_key: partition_key.to_string(),
                        data: EMPTY_ARRAY.to_vec(),
                    });
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

                    let contract = TcpContract::DeleteRows {
                        table_name: data.table_data.table_name.to_string(),
                        rows: deleted_rows,
                    };

                    result.push(contract);
                }
            }

            TcpContractsToSend::Multiple(result)
        }
        SyncEvent::DeleteTable(data) => {
            let contract = TcpContract::InitTable {
                table_name: data.table_data.table_name.to_string(),
                data: EMPTY_ARRAY.to_vec(),
            };

            TcpContractsToSend::Single(contract)
        }
    }
}
