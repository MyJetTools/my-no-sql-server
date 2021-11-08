use crate::{
    db::DbRow,
    db_json_entity::{DbJsonEntity, JsonTimeStamp},
};

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
    pub fn to_db_row(&self, time_stamp: &JsonTimeStamp) -> Result<DbRow, GrpcContractConvertError> {
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

    fn parse_as_json(&self, time_stamp: &JsonTimeStamp) -> Result<DbRow, GrpcContractConvertError> {
        let db_entity = DbJsonEntity::parse(self.content.as_ref())?;

        Ok(db_entity.to_db_row(time_stamp))
    }
}
