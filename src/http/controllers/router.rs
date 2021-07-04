use hyper::{Body, Method, Request, Response, Result, StatusCode};
use rand::Rng;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::app::AppServices;
use crate::http::http_ctx::HttpContext;
use crate::http::http_helpers;
use std::sync::Arc;

use super::{api, bulk, gc, logs, metrics, row, rows, status, tables};

pub async fn route_requests(req: Request<Body>, app: Arc<AppServices>) -> Result<Response<Body>> {
    let path = req.uri().path().to_lowercase();

    if path.starts_with("/logs/table") {
        return http_helpers::get_http_response(logs::get_by_table(app.as_ref(), &path).await);
    }

    let api_response_result = match (req.method(), path.as_str()) {
        (&Method::GET, "/api/isalive") => Some(api::is_alive()),
        (&Method::GET, "/api/status") => Some(status::get(app.as_ref()).await),
        (&Method::GET, "/metrics") => Some(metrics::get(app.as_ref())),
        (&Method::GET, "/logs") => Some(logs::get(app.as_ref()).await),

        (&Method::GET, "/tables/list") => Some(tables::list_of_tables(app.as_ref()).await),
        (&Method::POST, "/tables/create") => {
            Some(tables::create_table(HttpContext::new(req), app.as_ref()).await)
        }
        (&Method::POST, "/tables/createifnotexists") => {
            Some(tables::create_table_if_not_exists(HttpContext::new(req), app.as_ref()).await)
        }
        (&Method::DELETE, "/tables/clean") => {
            Some(tables::clean(HttpContext::new(req), app.as_ref()).await)
        }

        (&Method::DELETE, "/tables/updatepersist") => {
            Some(tables::update_persist(HttpContext::new(req), app.as_ref()).await)
        }

        (&Method::DELETE, "/tables/partitionscount") => {
            Some(tables::get_partitions_count(HttpContext::new(req), app.as_ref()).await)
        }

        (&Method::GET, "/row") => Some(row::get_rows(HttpContext::new(req), app.as_ref()).await),

        (&Method::PUT, "/row/replace") => {
            Some(row::replace(HttpContext::new(req), app.as_ref()).await)
        }

        (&Method::POST, "/row/insert") => {
            Some(row::insert(HttpContext::new(req), app.as_ref()).await)
        }
        (&Method::POST, "/row/insertorreplace") => {
            Some(row::insert_or_replace(HttpContext::new(req), app.as_ref()).await)
        }

        (&Method::POST, "/rows/singlepartitionmultiplerows") => Some(
            rows::get_single_partition_multiple_rows(HttpContext::new(req), app.as_ref()).await,
        ),

        (&Method::GET, "/rows/highestrowandbelow") => {
            Some(rows::get_highest_row_and_below(HttpContext::new(req), app.as_ref()).await)
        }

        (&Method::POST, "/bulk/insertorreplace") => {
            Some(bulk::insert_or_replace(HttpContext::new(req), app.as_ref()).await)
        }

        (&Method::POST, "/bulk/cleanandbulkinsert") => {
            Some(bulk::clean_and_bulk_insert(HttpContext::new(req), app.as_ref()).await)
        }

        (&Method::POST, "/bulk/delete") => {
            Some(bulk::bulk_delete(HttpContext::new(req), app.as_ref()).await)
        }

        (&Method::POST, "/garbagecollector/cleanandkeepmaxpartitions") => Some(
            gc::clean_and_keep_max_partitions_amount(HttpContext::new(req), app.as_ref()).await,
        ),

        (&Method::POST, "/garbagecollector/cleanandkeepmaxrecords") => {
            Some(gc::clean_and_keep_max_records(HttpContext::new(req), app.as_ref()).await)
        }

        _ => None,
    };

    if let Some(api_resp) = api_response_result {
        return http_helpers::get_http_response(api_resp);
    }

    if path == "/" {
        return get_index_page_content();
    }

    if path.starts_with("/swagger") {
        return serve_file(path.as_str()).await;
    }

    if path.starts_with("/css") {
        return serve_file_with_content_type(path.as_str(), "text/css").await;
    }

    if path.starts_with("/js") {
        return serve_file_with_content_type(path.as_str(), "text/javascript").await;
    }

    return Ok(not_found("Not Found".to_string()));
}

fn not_found(message: String) -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(message.into())
        .unwrap()
}

async fn get_file(filename: &str) -> std::io::Result<Vec<u8>> {
    let filename = format!("./wwwroot{}", filename);

    let mut file = File::open(&filename).await?;

    let mut result: Vec<u8> = Vec::new();

    loop {
        let res = file.read_buf(&mut result).await?;

        if res == 0 {
            break;
        }
    }

    return Ok(result);
}

async fn serve_file(filename: &str) -> Result<Response<Body>> {
    // Serve a file by asynchronously reading it by chunks using tokio-util crate.

    match get_file(filename).await {
        Ok(file) => {
            let body = Body::from(file);
            return Ok(Response::new(body));
        }
        Err(err) => {
            let msg = format!("Error handing file: {:?}. Filename: {}.", err, filename);
            Ok(not_found(msg))
        }
    }
}

async fn serve_file_with_content_type(
    filename: &str,
    content_type: &str,
) -> Result<Response<Body>> {
    // Serve a file by asynchronously reading it by chunks using tokio-util crate.

    match get_file(filename).await {
        Ok(content) => {
            let resp = Response::builder()
                .header("Content-Type", content_type)
                .body(Body::from(content))
                .unwrap();
            return Ok(resp);
        }
        Err(err) => {
            let msg = format!("Error handing file: {:?}. Filename: {}.", err, filename);
            Ok(not_found(msg))
        }
    }
}

fn get_index_page_content() -> Result<Response<Body>> {
    let mut rng = rand::thread_rng();

    let rnd: u64 = rng.gen();

    let content = format!(
        r###"<html><head><title>RUST MyNoSQLServer</title>
        <link href="/css/bootstrap.css" rel="stylesheet" type="text/css" />
        <link href="/css/site.css" rel="stylesheet" type="text/css" />
        <script src="/js/jquery.js"></script><script src="/js/app.js?ver={rnd}"></script>
        </head><body></body></html>"###,
        rnd = rnd
    );

    let result = Response::builder()
        .header("Content-Type", "text/html")
        .body(Body::from(content))
        .unwrap();

    Ok(result)
}
