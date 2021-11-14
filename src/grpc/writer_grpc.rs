use super::server::MyNoSqlServerWriterGrpcSerice;
use crate::db_json_entity::JsonTimeStamp;
use crate::mynosqlserver_grpc::writer_server::Writer;
use crate::mynosqlserver_grpc::*;
use futures_core::Stream;
use std::pin::Pin;
use tonic::Status;

const DEFAULT_SYNC_PERIOD: crate::db_sync::DataSynchronizationPeriod =
    crate::db_sync::DataSynchronizationPeriod::Sec5;

const OK_GRPC_RESPONSE: i32 = 0;
const TABLE_NOT_FOUND_GRPC_RESPONSE: i32 = 1;
const DB_ROW_NOT_FOUND_GRPC_RESPONSE: i32 = 2;

#[tonic::async_trait]
impl Writer for MyNoSqlServerWriterGrpcSerice {
    type GetRowsStream = Pin<
        Box<
            dyn Stream<Item = Result<TableEntityTransportGrpcContract, Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    async fn create_table_if_not_exists(
        &self,
        request: tonic::Request<CreateTableIfNotExistsGrpcRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let request = request.into_inner();

        let attr = crate::operations::transaction_attributes::create(
            self.app.as_ref(),
            DEFAULT_SYNC_PERIOD,
        );

        crate::db_operations::write::table::create_if_not_exist(
            self.app.as_ref(),
            &request.table_name,
            request.persist_table,
            None,
            Some(attr),
        )
        .await;

        return Ok(tonic::Response::new(()));
    }

    async fn set_table_attributes(
        &self,
        request: tonic::Request<SetTableAttributesGrpcRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let request = request.into_inner();

        let db_table =
            crate::db_operations::read::table::get(self.app.as_ref(), &request.table_name)
                .await
                .unwrap();

        let persist = db_table.get_persist().await;

        let attr = crate::operations::transaction_attributes::create(
            self.app.as_ref(),
            DEFAULT_SYNC_PERIOD,
        );

        let max_partitions_amount =
            if let Some(max_partitions_amount) = request.max_partitions_amount {
                Some(max_partitions_amount as usize)
            } else {
                None
            };

        crate::db_operations::write::table::set_table_attrubutes(
            self.app.as_ref(),
            db_table,
            persist,
            max_partitions_amount,
            Some(attr),
        )
        .await;

        return Ok(tonic::Response::new(()));
    }

    async fn get_rows(
        &self,
        request: tonic::Request<GetEntitiesGrpcRequest>,
    ) -> Result<tonic::Response<Self::GetRowsStream>, tonic::Status> {
        let request = request.into_inner();

        let db_table =
            crate::db_operations::read::table::get(self.app.as_ref(), &request.table_name)
                .await
                .unwrap();

        let partition_key = request.partition_key.as_ref();
        let row_key = request.row_key.as_ref();

        let limit = if let Some(limit) = request.limit {
            Some(limit as usize)
        } else {
            None
        };

        let skip = if let Some(skip) = request.skip {
            Some(skip as usize)
        } else {
            None
        };

        let (tx, rx) = tokio::sync::mpsc::channel(4);

        let date_time = JsonTimeStamp::now();

        let db_rows = crate::db_operations::read::get_rows_as_vec::execute(
            db_table.as_ref(),
            partition_key,
            row_key,
            limit,
            skip,
            &date_time,
        )
        .await;

        tokio::spawn(async move {
            if let Some(db_rows) = db_rows {
                for db_row in db_rows {
                    let grpc = TableEntityTransportGrpcContract {
                        content_type: 0,
                        content: db_row.data.clone(),
                    };

                    tx.send(Ok(grpc)).await.unwrap();
                }
            }
        });

        Ok(tonic::Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }

    async fn get_row(
        &self,
        request: tonic::Request<GetEntityGrpcRequest>,
    ) -> Result<tonic::Response<GetDbRowGrpcResponse>, tonic::Status> {
        let request = request.into_inner();

        let db_table =
            crate::db_operations::read::table::get(self.app.as_ref(), &request.table_name).await;

        if let Err(_) = db_table {
            let result = GetDbRowGrpcResponse {
                response_code: TABLE_NOT_FOUND_GRPC_RESPONSE,
                entity: None,
            };

            return Ok(tonic::Response::new(result));
        }

        let db_table = db_table.unwrap();

        let date_time = JsonTimeStamp::now();

        let db_row = crate::db_operations::read::get_rows_as_vec::get_as_partition_key_and_row_key(
            db_table.as_ref(),
            &request.partition_key,
            &request.row_key,
            &date_time,
        )
        .await;

        if db_row.is_none() {
            let result = GetDbRowGrpcResponse {
                response_code: DB_ROW_NOT_FOUND_GRPC_RESPONSE,
                entity: None,
            };
            return Ok(tonic::Response::new(result));
        }

        let entity = TableEntityTransportGrpcContract {
            content_type: 0,
            content: db_row.unwrap().data.clone(),
        };

        let result = GetDbRowGrpcResponse {
            response_code: OK_GRPC_RESPONSE,
            entity: Some(entity),
        };

        Ok(tonic::Response::new(result))
    }

    async fn post_transaction_actions(
        &self,
        request: tonic::Request<TransactionPayloadGrpcRequest>,
    ) -> Result<tonic::Response<TransactionGrpcResponse>, tonic::Status> {
        println!("PostTransaction");
        let request = request.into_inner();

        let transaction_id = if let Some(transaction_id) = &request.transaction_id {
            transaction_id.to_string()
        } else {
            self.app.active_transactions.issue_new().await
        };

        let mut events = Vec::new();
        let now = JsonTimeStamp::now();
        for action in &request.actions {
            let action = super::models::deserializer::deserialize(
                action.transaction_type.into(),
                &action.payload,
                &now,
            )
            .unwrap();

            events.push(action);
        }

        self.app
            .active_transactions
            .add_events(&transaction_id, events)
            .await;

        if request.commit {
            let attr = crate::operations::transaction_attributes::create(
                self.app.as_ref(),
                crate::db_sync::DataSynchronizationPeriod::Sec1,
            );
            crate::db_operations::transactions::commit(
                self.app.as_ref(),
                &transaction_id,
                attr,
                &now,
            )
            .await
            .unwrap();
        }

        let result = TransactionGrpcResponse {
            result: 0,
            id: transaction_id,
        };

        println!("PostTransaction Done");
        return Ok(tonic::Response::new(result));
    }

    async fn cancel_transaction(
        &self,
        request: tonic::Request<CancelTransactionGrpcRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        println!("CancelTransaction");
        let request = request.into_inner();

        self.app.active_transactions.remove(&request.id).await;

        return Ok(tonic::Response::new(()));
    }
}

impl Into<TransactionType> for i32 {
    fn into(self) -> TransactionType {
        match self {
            0 => TransactionType::CleanTable,
            1 => TransactionType::DeletePartitions,
            2 => TransactionType::DeleteRows,
            3 => TransactionType::InsertOrReplaceEntities,
            _ => {
                panic!("Invalid transaction type {}", self)
            }
        }
    }
}
