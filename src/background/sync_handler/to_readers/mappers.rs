use my_no_sql_tcp_shared::{DeleteRowTcpContract, TcpContract};

use crate::{db::read_as_json::DbEntityAsJsonArray, db_sync::SyncEvent, json::consts::EMPTY_ARRAY};

pub enum TcpContractsToSend {
    None,
    Single(TcpContract),
    Multiple(Vec<TcpContract>),
}

pub fn into_tcp_contract(event: &SyncEvent) -> TcpContractsToSend {
    match event {
        SyncEvent::UpdateTableAttributes {
            table: _,
            attr: _,
            table_is_just_created: _,
            persist: _,
            max_partitions_amount: _,
        } => TcpContractsToSend::None,
        SyncEvent::InitTable(state) => {
            let result = TcpContract::InitTable {
                table_name: state.table.name.to_string(),
                data: state.as_raw_bytes(),
            };

            TcpContractsToSend::Single(result)
        }
        SyncEvent::InitPartitions(state) => {
            let mut result = Vec::new();

            for (partition_key, data) in &state.partitions_to_update {
                let contract = TcpContract::InitPartition {
                    partition_key: partition_key.to_string(),
                    table_name: state.table.name.to_string(),
                    data: if let Some(data) = data {
                        data.content.as_json_array()
                    } else {
                        EMPTY_ARRAY.to_vec()
                    },
                };

                result.push(contract);
            }

            TcpContractsToSend::Multiple(result)
        }
        SyncEvent::UpdateRows(state) => {
            let result = TcpContract::UpdateRows {
                table_name: state.table.name.to_string(),
                data: state.updated_rows_by_partition.as_json_array(),
            };
            TcpContractsToSend::Single(result)
        }
        SyncEvent::Delete(state) => {
            let mut result = Vec::new();

            if let Some(deleted_partitions) = &state.deleted_partitions {
                for (partition_key, _) in deleted_partitions {
                    result.push(TcpContract::InitPartition {
                        table_name: state.table.name.to_string(),
                        partition_key: partition_key.to_string(),
                        data: EMPTY_ARRAY.to_vec(),
                    });
                }
            }

            if let Some(deleted_rows) = &state.deleted_rows {
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
                        table_name: state.table.name.to_string(),
                        rows: deleted_rows,
                    };

                    result.push(contract);
                }
            }

            TcpContractsToSend::Multiple(result)
        }
        SyncEvent::DeleteTable { table, attr: _ } => {
            let contract = TcpContract::InitTable {
                table_name: table.name.to_string(),
                data: EMPTY_ARRAY.to_vec(),
            };

            TcpContractsToSend::Single(contract)
        }
    }
}
