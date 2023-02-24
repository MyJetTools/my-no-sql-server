use std::{collections::BTreeMap, sync::Arc};

use super::{
    table_entity_transport_grpc_contract::TableEntityTransportGrpcContract,
    GrpcContractConvertError,
};
use my_no_sql_core::{db::DbRow, db_json_entity::JsonTimeStamp};
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

    pub fn to_db_rows(
        &self,
        now: &JsonTimeStamp,
    ) -> Result<BTreeMap<String, Vec<Arc<DbRow>>>, GrpcContractConvertError> {
        let mut result = BTreeMap::new();

        for entity in &self.entities {
            let db_rows = entity.to_db_rows(&now)?;

            for db_row in db_rows {
                if !result.contains_key(&db_row.partition_key) {
                    result.insert(db_row.partition_key.to_string(), Vec::new());
                }

                result.get_mut(&db_row.partition_key).unwrap().push(db_row);
            }
        }

        Ok(result)
    }
}
