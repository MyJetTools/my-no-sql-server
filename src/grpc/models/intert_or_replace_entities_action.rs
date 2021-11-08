use super::table_entity_transport_grpc_contract::TableEntityTransportGrpcContract;
use prost::DecodeError;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertOrReplaceEntitiesTransactionActionGrpcModel {
    #[prost(string, tag = "1")]
    pub table_name: String,
    #[prost(message, repeated, tag = "2")]
    pub entities: Vec<TableEntityTransportGrpcContract>,
}

impl InsertOrReplaceEntitiesTransactionActionGrpcModel {
    pub fn deserialize(payload: &[u8]) -> Result<Self, DecodeError> {
        prost::Message::decode(payload)
    }
}
