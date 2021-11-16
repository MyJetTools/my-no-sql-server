use crate::{
    db_json_entity::DbEntityParseFail, db_operations::DbOperationError, json::JsonParseError,
};

use my_http_utils::{HttpFailResult, WebContentType};
use serde::{Deserialize, Serialize};

const TABLE_ALREADY_EXISTS_ERR: i16 = -1;
const TABLE_NOT_FOUND_ERR: i16 = -2;
const RECORD_ALREADY_EXISTS_ERR: i16 = -3;
const REQUIERED_ENTITY_FIELD_IS_MISSING_ERR: i16 = -4;
const JSON_PARSE_ERR: i16 = -5;

#[derive(Serialize, Deserialize, Debug)]
struct HttpErrorModel {
    pub code: i16,
    pub message: String,
}

impl From<DbOperationError> for HttpFailResult {
    fn from(src: DbOperationError) -> Self {
        match src {
            DbOperationError::TableAlreadyExists => {
                let err_model = HttpErrorModel {
                    code: TABLE_ALREADY_EXISTS_ERR,
                    message: format!("Table already exists"),
                };
                let content = serde_json::to_vec(&err_model).unwrap();

                HttpFailResult {
                    content_type: WebContentType::Json,
                    status_code: 400,
                    content,
                    metric_it: true,
                }
            }
            DbOperationError::TableNotFound(table_name) => {
                let err_model = HttpErrorModel {
                    code: TABLE_NOT_FOUND_ERR,
                    message: format!("Table '{}' not found", table_name),
                };
                let content = serde_json::to_vec(&err_model).unwrap();

                HttpFailResult {
                    content_type: WebContentType::Json,
                    status_code: 400,
                    content,
                    metric_it: true,
                }
            }
            DbOperationError::RecordNotFound => HttpFailResult {
                content_type: WebContentType::Json,
                status_code: 404,
                content: format!("Record not found").into_bytes(),
                metric_it: false,
            },
            DbOperationError::OptimisticConcurencyUpdateFails => HttpFailResult {
                content_type: WebContentType::Json,
                status_code: 409,
                content: format!("Record is changed").into_bytes(),
                metric_it: false,
            },
            DbOperationError::RecordAlreadyExists => {
                let err_model = HttpErrorModel {
                    code: RECORD_ALREADY_EXISTS_ERR,
                    message: format!("Record already exists"),
                };
                let content = serde_json::to_vec(&err_model).unwrap();

                HttpFailResult {
                    content_type: WebContentType::Json,
                    status_code: 400,
                    content,
                    metric_it: false,
                }
            }
            DbOperationError::TimeStampFieldRequires => {
                let err_model = HttpErrorModel {
                    code: REQUIERED_ENTITY_FIELD_IS_MISSING_ERR,
                    message: format!("Timestamp field requires"),
                };

                let content = serde_json::to_vec(&err_model).unwrap();
                HttpFailResult {
                    content_type: WebContentType::Text,
                    status_code: 400,
                    content,
                    metric_it: true,
                }
            }
        }
    }
}

impl From<JsonParseError> for HttpFailResult {
    fn from(value: JsonParseError) -> Self {
        let err_model = HttpErrorModel {
            code: JSON_PARSE_ERR,
            message: value.to_string(),
        };

        let content = serde_json::to_vec(&err_model).unwrap();

        Self {
            content_type: WebContentType::Json,
            status_code: 400,
            content,
            metric_it: true,
        }
    }
}

impl From<DbEntityParseFail> for HttpFailResult {
    fn from(src: DbEntityParseFail) -> Self {
        match src {
            DbEntityParseFail::FieldPartitionKeyIsRequired => {
                let err_model = HttpErrorModel {
                    code: REQUIERED_ENTITY_FIELD_IS_MISSING_ERR,
                    message: format!("PartitionKey field is required"),
                };

                let content = serde_json::to_vec(&err_model).unwrap();

                Self {
                    content_type: WebContentType::Json,
                    status_code: 400,
                    content,
                    metric_it: true,
                }
            }
            DbEntityParseFail::FieldRowKeyIsRequired => {
                let err_model = HttpErrorModel {
                    code: REQUIERED_ENTITY_FIELD_IS_MISSING_ERR,
                    message: format!("RowKey field is required"),
                };

                let content = serde_json::to_vec(&err_model).unwrap();

                Self {
                    content_type: WebContentType::Json,
                    status_code: 400,
                    content,
                    metric_it: true,
                }
            }

            DbEntityParseFail::JsonParseError(json_parse_error) => {
                HttpFailResult::from(json_parse_error)
            }
        }
    }
}
