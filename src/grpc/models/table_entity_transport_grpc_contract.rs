use crate::{
    db::DbRow,
    db_json_entity::{DbJsonEntity, JsonTimeStamp},
};
use std::sync::Arc;

use super::GrpcContractConvertError;

#[derive(Clone, PartialEq, Debug, ::prost::Enumeration)]
#[repr(i32)]
pub enum GrpcContentType {
    Json = 0,
    Protobuf = 1,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TableEntityTransportGrpcContract {
    #[prost(enumeration = "GrpcContentType", tag = "1")]
    pub content_type: i32,
    #[prost(bytes, tag = "2")]
    pub content: Vec<u8>,
}

impl TableEntityTransportGrpcContract {
    pub fn to_db_rows(
        &self,
        time_stamp: &JsonTimeStamp,
    ) -> Result<Vec<Arc<DbRow>>, GrpcContractConvertError> {
        let result = GrpcContentType::from_i32(self.content_type).unwrap();

        match result {
            GrpcContentType::Json => {
                return self.parse_as_json(time_stamp);
            }
            GrpcContentType::Protobuf => {
                todo!("Not Implemented")
            }
        }
    }

    fn parse_as_json(
        &self,
        time_stamp: &JsonTimeStamp,
    ) -> Result<Vec<Arc<DbRow>>, GrpcContractConvertError> {
        let result = DbJsonEntity::parse_as_vec(self.content.as_ref(), time_stamp)?;

        Ok(result)
    }
}
