use my_no_sql_tcp_shared::{DeleteRowTcpContract, TcpContract};

use crate::{db_sync::SyncEvent, json::consts::EMPTY_ARRAY};

pub enum TcpPayloadToSend {
    FirstInit(TcpContract),
    Single(Vec<u8>),
    Multiple(Vec<Vec<u8>>),
}

impl TcpPayloadToSend {
    pub async fn parse_from(sync_event: &SyncEvent) -> Option<TcpPayloadToSend> {
        match sync_event {
            SyncEvent::TableFirstInit(sync_data) => {
                let table_snapshot = sync_data.db_table.get_table_snapshot().await;

                let data = table_snapshot.as_json_array().build();

                let tcp_contract = TcpContract::InitTable {
                    table_name: sync_data.db_table.name.to_string(),
                    data,
                };

                TcpPayloadToSend::FirstInit(tcp_contract).into()
            }
            SyncEvent::UpdateTableAttributes(_) => None,
            SyncEvent::InitTable(sync_data) => {
                let data = sync_data.table_snapshot.as_json_array().build();

                let result = TcpContract::InitTable {
                    table_name: sync_data.table_data.table_name.to_string(),
                    data,
                };

                TcpPayloadToSend::Single(result.serialize()).into()
            }
            SyncEvent::InitPartitions(data) => {
                let mut result = Vec::new();

                for (partition_key, snapshot) in &data.partitions_to_update {
                    let contract = TcpContract::InitPartition {
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

                    result.push(contract.serialize());
                }

                TcpPayloadToSend::Multiple(result).into()
            }
            SyncEvent::UpdateRows(data) => {
                let result = TcpContract::UpdateRows {
                    table_name: data.table_data.table_name.to_string(),
                    data: data.rows_by_partition.as_json_array().build(),
                };
                TcpPayloadToSend::Single(result.serialize()).into()
            }
            SyncEvent::DeleteRows(data) => {
                let mut result = Vec::new();

                if let Some(deleted_partitions) = &data.deleted_partitions {
                    for (partition_key, _) in deleted_partitions {
                        result.push(
                            TcpContract::InitPartition {
                                table_name: data.table_data.table_name.to_string(),
                                partition_key: partition_key.to_string(),
                                data: EMPTY_ARRAY.to_vec(),
                            }
                            .serialize(),
                        );
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

                        result.push(contract.serialize());
                    }
                }

                TcpPayloadToSend::Multiple(result).into()
            }
            SyncEvent::DeleteTable(data) => {
                let contract = TcpContract::InitTable {
                    table_name: data.table_data.table_name.to_string(),
                    data: EMPTY_ARRAY.to_vec(),
                };

                TcpPayloadToSend::Single(contract.serialize()).into()
            }
        }
    }
}
