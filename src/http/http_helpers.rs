use std::sync::Arc;

use crate::{
    app::AppServices,
    db::{DbRow, FailOperationResult, OperationResult},
    db_transactions::{DataSynchronizationPeriod, TransactionAttributes},
};

use hyper::{Body, Response, Result};

fn get_ok_response(operation_result: OperationResult) -> Response<Body> {
    return match operation_result {
        OperationResult::Ok => Response::builder()
            .header("Content-Type", "text/plain")
            .status(200)
            .body(Body::from("OK"))
            .unwrap(),
        OperationResult::Json { json } => Response::builder()
            .header("Content-Type", "application/json")
            .status(200)
            .body(Body::from(json))
            .unwrap(),
        OperationResult::Text { text } => Response::builder()
            .header("Content-Type", "text/plain")
            .status(200)
            .body(Body::from(text))
            .unwrap(),

        OperationResult::Html { title, body } => Response::builder()
            .header("Content-Type", "text/html")
            .status(200)
            .body(Body::from(compile_html(title, body)))
            .unwrap(),
        OperationResult::Number { value } => Response::builder()
            .header("Content-Type", "text/plain; charset=utf-8 ")
            .status(200)
            .body(Body::from(format!("{}", value)))
            .unwrap(),

        OperationResult::Rows { rows } => Response::builder()
            .header("Content-Type", "application/json")
            .status(200)
            .body(Body::from(to_json_array(rows)))
            .unwrap(),

        OperationResult::Row { row } => Response::builder()
            .header("Content-Type", "application/json")
            .status(200)
            .body(Body::from(row.data.clone()))
            .unwrap(),
    };
}

pub fn to_json_array(db_rows: Option<Vec<Arc<DbRow>>>) -> Vec<u8> {
    if db_rows.is_none() {
        return vec![
            crate::json::consts::OPEN_ARRAY,
            crate::json::consts::CLOSE_ARRAY,
        ];
    }

    let mut json = Vec::new();

    let db_rows = db_rows.unwrap();

    for db_row in db_rows.as_slice() {
        if json.len() == 0 {
            json.push(crate::json::consts::OPEN_ARRAY);
        } else {
            json.push(crate::json::consts::COMMA);
        }

        json.extend(db_row.data.as_slice());
    }

    json.push(crate::json::consts::CLOSE_ARRAY);

    return json;
}

fn get_fail_response(fail_result: FailOperationResult) -> Response<Body> {
    return match fail_result {
        FailOperationResult::TableAlreadyExist { table_name } => Response::builder()
            .header("Content-Type", "text/plain")
            .status(200)
            .body(Body::from(format!("Table '{}' already exists", table_name)))
            .unwrap(),

        FailOperationResult::FieldPartitionKeyIsRequired => Response::builder()
            .header("Content-Type", "text/plain")
            .status(400)
            .body(Body::from(format!(
                "Field partitionKey is required to execute operation"
            )))
            .unwrap(),

        FailOperationResult::JsonParseError(err) => Response::builder()
            .header("Content-Type", "text/plain")
            .status(500)
            .body(Body::from(err.to_string()))
            .unwrap(),

        _ => Response::builder()
            .header("Content-Type", "text/plain")
            .status(500)
            .body(Body::from(format!("{:?}", fail_result)))
            .unwrap(),
    };
}

pub fn get_http_response(
    src: std::result::Result<OperationResult, FailOperationResult>,
) -> Result<Response<Body>> {
    let response = match src {
        Ok(ok_result) => get_ok_response(ok_result),
        Err(fail_result) => get_fail_response(fail_result),
    };

    return Ok(response);
}

pub fn create_transaction_attributes(
    app: &AppServices,
    sync_period: DataSynchronizationPeriod,
) -> TransactionAttributes {
    let locations = vec![app.settings.location.to_string()];
    TransactionAttributes {
        locations,
        event_source: crate::db_transactions::EventSource::ClientRequest,
        headers: None, //TODO - Enable Headers,
        sync_period,
    }
}

fn compile_html(title: String, body: String) -> String {
    format!(
        r###"<html><head><title>{ver} MyNoSQLServer {title}</title>
        <link href="/css/bootstrap.css" rel="stylesheet" type="text/css" />
        </head><body>{body}</body></html>"###,
        ver = crate::app::APP_VERSION,
        title = title,
        body = body
    )
}
