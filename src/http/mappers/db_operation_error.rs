use crate::{
    db_json_entity::DbEntityParseFail, db_operations::DbOperationError, json::JsonParseError,
};

use my_http_server::{HttpFailResult, WebContentType};

use super::{OperationFailHttpContract, OperationFailReason};

pub const OPERATION_FAIL_HTTP_STATUS_CODE: u16 = 400;

impl From<DbOperationError> for HttpFailResult {
    fn from(src: DbOperationError) -> Self {
        match src {
            DbOperationError::TableAlreadyExists => {
                let err_model = OperationFailHttpContract {
                    reason: OperationFailReason::TableAlreadyExists,
                    message: format!("Table already exists"),
                };
                let content = serde_json::to_vec(&err_model).unwrap();

                HttpFailResult {
                    content_type: WebContentType::Json,
                    status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                    content,
                    write_telemetry: true,
                }
            }
            DbOperationError::TableNotFound(table_name) => {
                super::super::get_table::table_not_found_http_result(table_name.as_str())
            }
            DbOperationError::RecordNotFound => HttpFailResult {
                content_type: WebContentType::Json,
                status_code: 404,
                content: format!("Record not found").into_bytes(),
                write_telemetry: false,
            },
            DbOperationError::OptimisticConcurencyUpdateFails => HttpFailResult {
                content_type: WebContentType::Json,
                status_code: 409,
                content: format!("Record is changed").into_bytes(),
                write_telemetry: false,
            },
            DbOperationError::RecordAlreadyExists => {
                let err_model = OperationFailHttpContract {
                    reason: OperationFailReason::RecordAlreadyExists,
                    message: format!("Record already exists"),
                };
                let content = serde_json::to_vec(&err_model).unwrap();

                HttpFailResult {
                    content_type: WebContentType::Json,
                    status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                    content,
                    write_telemetry: false,
                }
            }
            DbOperationError::TimeStampFieldRequires => {
                let err_model = OperationFailHttpContract {
                    reason: OperationFailReason::RequieredEntityFieldIsMissing,
                    message: format!("Timestamp field requires"),
                };

                let content = serde_json::to_vec(&err_model).unwrap();
                HttpFailResult {
                    content_type: WebContentType::Text,
                    status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                    content,
                    write_telemetry: true,
                }
            }
            DbOperationError::TableNameValidationError(reason) => {
                let err_model = OperationFailHttpContract {
                    reason: OperationFailReason::RequieredEntityFieldIsMissing,
                    message: format!("Invalid table name: {}", reason),
                };

                let content = serde_json::to_vec(&err_model).unwrap();
                HttpFailResult {
                    content_type: WebContentType::Text,
                    status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                    content,
                    write_telemetry: true,
                }
            }
        }
    }
}

impl From<JsonParseError> for HttpFailResult {
    fn from(value: JsonParseError) -> Self {
        let err_model = OperationFailHttpContract {
            reason: OperationFailReason::JsonParseFail,
            message: value.to_string(),
        };

        let content = serde_json::to_vec(&err_model).unwrap();

        Self {
            content_type: WebContentType::Json,
            status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
            content,
            write_telemetry: true,
        }
    }
}

impl From<DbEntityParseFail> for HttpFailResult {
    fn from(src: DbEntityParseFail) -> Self {
        match src {
            DbEntityParseFail::FieldPartitionKeyIsRequired => {
                let err_model = OperationFailHttpContract {
                    reason: OperationFailReason::RequieredEntityFieldIsMissing,
                    message: format!("PartitionKey field is required"),
                };

                let content = serde_json::to_vec(&err_model).unwrap();

                Self {
                    content_type: WebContentType::Json,
                    status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                    content,
                    write_telemetry: true,
                }
            }
            DbEntityParseFail::FieldRowKeyIsRequired => {
                let err_model = OperationFailHttpContract {
                    reason: OperationFailReason::RequieredEntityFieldIsMissing,
                    message: format!("RowKey field is required"),
                };

                let content = serde_json::to_vec(&err_model).unwrap();

                Self {
                    content_type: WebContentType::Json,
                    status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                    content,
                    write_telemetry: true,
                }
            }

            DbEntityParseFail::JsonParseError(json_parse_error) => {
                HttpFailResult::from(json_parse_error)
            }
        }
    }
}
