use crate::{
    db_json_entity::DbEntityParseFail,
    db_operations::DbOperationError,
    http::{http_fail::HttpFailResult, web_content_type::WebContentType},
    json::JsonParseError,
};

use serde::{Deserialize, Serialize};

const TABLE_ALREADY_EXISTS_ERR: i16 = -1;
const TABLE_NOT_FOUND_ERR: i16 = -2;
const RECORD_NOT_FOUND_ERR: i16 = -3;
const OPTIMISTIC_CONCURENCY_ERR: i16 = -4;
const RECORD_ALREADY_EXISTS_ERR: i16 = -5;
const FIELD_REQUIERS_ERR: i16 = -6;
const JSON_PARSE_ERR: i16 = -7;

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
                }
            }
            DbOperationError::RecordNotFound => {
                let err_model = HttpErrorModel {
                    code: RECORD_NOT_FOUND_ERR,
                    message: format!("Record not found"),
                };
                let content = serde_json::to_vec(&err_model).unwrap();

                HttpFailResult {
                    content_type: WebContentType::Json,
                    status_code: 400,
                    content,
                }
            }
            DbOperationError::OptimisticConcurencyUpdateFails => {
                let err_model = HttpErrorModel {
                    code: OPTIMISTIC_CONCURENCY_ERR,
                    message: format!("Record is changed"),
                };
                let content = serde_json::to_vec(&err_model).unwrap();

                HttpFailResult {
                    content_type: WebContentType::Json,
                    status_code: 400,
                    content,
                }
            }
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
                }
            }
            DbOperationError::TimeStampFieldRequires => {
                let err_model = HttpErrorModel {
                    code: FIELD_REQUIERS_ERR,
                    message: format!("Timestamp field requires"),
                };

                let content = serde_json::to_vec(&err_model).unwrap();
                HttpFailResult {
                    content_type: WebContentType::Text,
                    status_code: 400,
                    content,
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
        }
    }
}

impl From<DbEntityParseFail> for HttpFailResult {
    fn from(src: DbEntityParseFail) -> Self {
        match src {
            DbEntityParseFail::FieldPartitionKeyIsRequired => {
                let err_model = HttpErrorModel {
                    code: FIELD_REQUIERS_ERR,
                    message: format!("PartitionKey field is required"),
                };

                let content = serde_json::to_vec(&err_model).unwrap();

                Self {
                    content_type: WebContentType::Json,
                    status_code: 400,
                    content,
                }
            }
            DbEntityParseFail::FieldRowKeyIsRequired => {
                let err_model = HttpErrorModel {
                    code: FIELD_REQUIERS_ERR,
                    message: format!("RowKey field is required"),
                };

                let content = serde_json::to_vec(&err_model).unwrap();

                Self {
                    content_type: WebContentType::Json,
                    status_code: 400,
                    content,
                }
            }

            DbEntityParseFail::JsonParseError(json_parse_error) => {
                HttpFailResult::from(json_parse_error)
            }
        }
    }
}
