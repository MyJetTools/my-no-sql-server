use crate::db_operations::DbOperationError;

use my_http_server::{HttpFailResult, WebContentType};
use my_no_sql_sdk::core::db_json_entity::DbEntityParseFail;
use my_no_sql_sdk::core::my_json::json_reader::JsonParseError;

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
                    write_to_log: true,
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
                write_to_log: false,
            },
            DbOperationError::ApplicationIsNotInitializedYet => HttpFailResult {
                content_type: WebContentType::Json,
                status_code: 503,
                content: format!("Application is not initialized yet").into_bytes(),
                write_telemetry: false,
                write_to_log: false,
            },
            DbOperationError::OptimisticConcurrencyUpdateFails => HttpFailResult {
                content_type: WebContentType::Json,
                status_code: 409,
                content: format!("Record is changed").into_bytes(),
                write_telemetry: false,
                write_to_log: false,
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
                    write_to_log: false,
                }
            }
            DbOperationError::TimeStampFieldRequires => {
                let err_model = OperationFailHttpContract {
                    reason: OperationFailReason::RequiredEntityFieldIsMissing,
                    message: format!("Timestamp field requires"),
                };

                let content = serde_json::to_vec(&err_model).unwrap();
                HttpFailResult {
                    content_type: WebContentType::Text,
                    status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                    content,
                    write_telemetry: true,
                    write_to_log: true,
                }
            }
            DbOperationError::TableNameValidationError(reason) => {
                let err_model = OperationFailHttpContract {
                    reason: OperationFailReason::RequiredEntityFieldIsMissing,
                    message: format!("Invalid table name: {}", reason),
                };

                let content = serde_json::to_vec(&err_model).unwrap();
                HttpFailResult {
                    content_type: WebContentType::Text,
                    status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                    content,
                    write_telemetry: true,
                    write_to_log: true,
                }
            }
            DbOperationError::DbEntityParseFail(src) => {
                let err_model = OperationFailHttpContract {
                    reason: OperationFailReason::JsonParseFail,
                    message: format!("{:?}", src),
                };

                let content = serde_json::to_vec(&err_model).unwrap();

                HttpFailResult {
                    content_type: WebContentType::Json,
                    status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                    content,
                    write_telemetry: true,
                    write_to_log: true,
                }
            }
        }
    }
}

pub fn from_json_parse_error_to_http_result(value: JsonParseError) -> HttpFailResult {
    let err_model = OperationFailHttpContract {
        reason: OperationFailReason::JsonParseFail,
        message: value.to_string(),
    };

    let content = serde_json::to_vec(&err_model).unwrap();

    HttpFailResult {
        content_type: WebContentType::Json,
        status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
        content,
        write_telemetry: true,
        write_to_log: true,
    }
}

pub fn from_db_entity_parse_fail_to_http_result(src: DbEntityParseFail) -> HttpFailResult {
    match src {
        DbEntityParseFail::FieldPartitionKeyIsRequired => {
            let err_model = OperationFailHttpContract {
                reason: OperationFailReason::RequiredEntityFieldIsMissing,
                message: format!("PartitionKey field is required"),
            };

            let content = serde_json::to_vec(&err_model).unwrap();

            HttpFailResult {
                content_type: WebContentType::Json,
                status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                content,
                write_telemetry: true,
                write_to_log: true,
            }
        }
        DbEntityParseFail::PartitionKeyIsTooLong => {
            let err_model = OperationFailHttpContract {
                reason: OperationFailReason::RequiredEntityFieldIsMissing,
                message: format!("PartitionKey is too long"),
            };

            let content = serde_json::to_vec(&err_model).unwrap();

            HttpFailResult {
                content_type: WebContentType::Json,
                status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                content,
                write_telemetry: true,
                write_to_log: true,
            }
        }
        DbEntityParseFail::FieldRowKeyIsRequired => {
            let err_model = OperationFailHttpContract {
                reason: OperationFailReason::RequiredEntityFieldIsMissing,
                message: format!("RowKey field is required"),
            };

            let content = serde_json::to_vec(&err_model).unwrap();

            HttpFailResult {
                content_type: WebContentType::Json,
                status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                content,
                write_telemetry: true,
                write_to_log: true,
            }
        }

        DbEntityParseFail::JsonParseError(json_parse_error) => {
            from_json_parse_error_to_http_result(json_parse_error)
        }
        DbEntityParseFail::FieldPartitionKeyCanNotBeNull => {
            let err_model = OperationFailHttpContract {
                reason: OperationFailReason::RequiredEntityFieldIsMissing,
                message: format!("PartitionKey can not be null"),
            };

            let content = serde_json::to_vec(&err_model).unwrap();

            HttpFailResult {
                content_type: WebContentType::Json,
                status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                content,
                write_telemetry: true,
                write_to_log: true,
            }
        }
        DbEntityParseFail::FieldRowKeyCanNotBeNull => {
            let err_model = OperationFailHttpContract {
                reason: OperationFailReason::RequiredEntityFieldIsMissing,
                message: format!("RowKey can not be null"),
            };

            let content = serde_json::to_vec(&err_model).unwrap();

            HttpFailResult {
                content_type: WebContentType::Json,
                status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
                content,
                write_telemetry: true,
                write_to_log: true,
            }
        }
    }
}
