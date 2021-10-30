use my_no_sql_tcp_shared::{DeleteRowTcpContract, TcpContract};

use crate::{db::read_as_json::DbEntityAsJsonArray, db_sync::SyncEvent, json::consts::EMPTY_ARRAY};

pub fn into_tcp_contract(event: &SyncEvent) -> Option<Vec<TcpContract>> {
    match event {
        SyncEvent::UpdateTableAttributes {
            table,
            attr,
            table_is_just_created,
            persist,
            max_partitions_amount,
        } => None,
        SyncEvent::InitTable(state) => {
            let result = TcpContract::InitTable {
                table_name: state.table.name.to_string(),
                data: state.get_snapshot(),
            };

            Some(vec![result])
        }
        SyncEvent::InitPartitions(state) => {
            let mut result = Vec::new();

            for (partition_key, data) in state.partitions_to_update {
                let contract = TcpContract::InitPartition {
                    partition_key,
                    table_name: state.table.name.to_string(),
                    data: if let Some(data) = data {
                        data.content.as_json_array(None)
                    } else {
                        EMPTY_ARRAY.to_vec()
                    },
                };

                result.push(contract);
            }

            Some(result)
        }
        SyncEvent::UpdateRows(state) => {
            let result = TcpContract::UpdateRows {
                table_name: state.table.name.to_string(),
                data: state.updated_rows_by_partition.as_json_array(None),
            };
            Some(vec![result])
        }
        SyncEvent::Delete(state) => {
            let mut result = Vec::new();

            if let Some(deleted_partitions) = state.deleted_partitions {
                for (partition_key, _) in deleted_partitions {
                    result.push(TcpContract::InitPartition {
                        table_name: state.table.name.to_string(),
                        partition_key,
                        data: EMPTY_ARRAY.to_vec(),
                    });
                }
            }

            if let Some(deleted_rows) = state.deleted_rows {
                for (partition_key, rows) in deleted_rows {
                    let mut deleted_rows = Vec::new();

                    for (row_key, row) in rows {
                        let contract = DeleteRowTcpContract {
                            partition_key: partition_key.to_string(),
                            row_key,
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

            return Some(result);
        }
        SyncEvent::DeleteTable { table, attr } => None,
    }
}
