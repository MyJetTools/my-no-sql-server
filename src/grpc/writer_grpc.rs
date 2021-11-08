use std::sync::Arc;

use crate::app::AppContext;
use crate::mynosqlserver_grpc::writer_server::Writer;
use crate::mynosqlserver_grpc::*;

#[derive(Clone)]
pub struct WriterGrpcSerice {
    pub app: Arc<AppContext>,
}

#[tonic::async_trait]
impl Writer for WriterGrpcSerice {
    async fn post_transaction_operations(
        &self,
        request: tonic::Request<TransactionPayloadGrpcRequest>,
    ) -> Result<tonic::Response<TransactionGrpcResponse>, tonic::Status> {
        let request = request.into_inner();

        let transaction_id = if let Some(transaction_id) = &request.transaction_id {
            transaction_id.to_string()
        } else {
            self.app.active_transactions.issue_new().await
        };

        let mut events = Vec::new();
        for action in &request.actions {
            let action = super::models::deserializer::deserialize(
                action.transaction_type.into(),
                &action.payload,
            )
            .unwrap();

            events.push(action);
        }

        self.app
            .active_transactions
            .add_events(&transaction_id, events)
            .await;

        let result = TransactionGrpcResponse {
            result: 0,
            id: transaction_id,
        };
        return Ok(tonic::Response::new(result));
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
