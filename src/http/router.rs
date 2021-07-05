use hyper::{Body, Method, Request};

use crate::app::AppServices;
use crate::http::http_ctx::HttpContext;
use std::sync::Arc;

use super::{
    controllers::{api, bulk, gc, logs, metrics, row, rows, status, tables, transactions},
    http_fail::HttpFailResult,
    http_ok::HttpOkResult,
    static_files,
};

pub async fn route_requests(
    req: Request<Body>,
    app: Arc<AppServices>,
) -> Result<HttpOkResult, HttpFailResult> {
    let path = req.uri().path().to_lowercase();

    match (req.method(), path.as_str()) {
        (&Method::GET, "/api/isalive") => {
            return api::is_alive();
        }
        (&Method::GET, "/api/status") => {
            return status::get(app.as_ref()).await;
        }
        (&Method::GET, "/metrics") => {
            return metrics::get(app.as_ref());
        }
        (&Method::GET, "/logs") => {
            return logs::get(app.as_ref()).await;
        }

        (&Method::GET, "/tables/list") => {
            return tables::list_of_tables(app.as_ref()).await;
        }
        (&Method::POST, "/tables/create") => {
            return tables::create_table(HttpContext::new(req), app.as_ref()).await;
        }
        (&Method::POST, "/tables/createifnotexists") => {
            return tables::create_table_if_not_exists(HttpContext::new(req), app.as_ref()).await;
        }
        (&Method::DELETE, "/tables/clean") => {
            return tables::clean(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::DELETE, "/tables/updatepersist") => {
            return tables::update_persist(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::DELETE, "/tables/partitionscount") => {
            return tables::get_partitions_count(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::GET, "/row") => {
            return row::get_rows(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::PUT, "/row/replace") => {
            return row::replace(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/row/insert") => {
            return row::insert(HttpContext::new(req), app.as_ref()).await;
        }
        (&Method::POST, "/row/insertorreplace") => {
            return row::insert_or_replace(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/rows/singlepartitionmultiplerows") => {
            return rows::get_single_partition_multiple_rows(HttpContext::new(req), app.as_ref())
                .await;
        }

        (&Method::GET, "/rows/highestrowandbelow") => {
            return rows::get_highest_row_and_below(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/bulk/insertorreplace") => {
            return bulk::insert_or_replace(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/bulk/cleanandbulkinsert") => {
            return bulk::clean_and_bulk_insert(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/bulk/delete") => {
            return bulk::bulk_delete(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/garbagecollector/cleanandkeepmaxpartitions") => {
            return gc::clean_and_keep_max_partitions_amount(HttpContext::new(req), app.as_ref())
                .await;
        }

        (&Method::POST, "/garbagecollector/cleanandkeepmaxrecords") => {
            return gc::clean_and_keep_max_records(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/transaction/start") => {
            return transactions::start(app.as_ref()).await;
        }

        (&Method::POST, "/transaction/append") => {
            return transactions::append(app.as_ref(), HttpContext::new(req)).await;
        }

        (&Method::POST, "/transaction/commit") => {
            return transactions::commit(app.as_ref(), HttpContext::new(req)).await;
        }

        (&Method::POST, "/transaction/cancel") => {
            return transactions::cancel(app.as_ref(), HttpContext::new(req)).await;
        }

        (&Method::GET, "/swagger") => {
            return static_files::serve_path("/swagger/index.html").await;
        }
        _ => {}
    };

    if path.starts_with("/logs/table") {
        return logs::get_by_table(app.as_ref(), &path).await;
    }

    if path.starts_with("/logs/process") {
        return logs::get_by_process(app.as_ref(), &path).await;
    }

    if path == "/" {
        return Ok(get_index_page_content(app.as_ref()));
    }

    if path.starts_with("/swagger") {
        return static_files::serve_path(path.as_str()).await;
    }

    if path.starts_with("/css") {
        return static_files::serve_path(path.as_str()).await;
    }

    if path.starts_with("/js") {
        return static_files::serve_path(path.as_str()).await;
    }

    return Err(HttpFailResult::as_not_found("Not Found".to_string()));
}

fn get_index_page_content(app: &AppServices) -> HttpOkResult {
    let content = format!(
        r###"<html><head><title>{} MyNoSQLServer</title>
        <link href="/css/bootstrap.css" rel="stylesheet" type="text/css" />
        <link href="/css/site.css" rel="stylesheet" type="text/css" />
        <script src="/js/jquery.js"></script><script src="/js/app.js?ver={rnd}"></script>
        </head><body></body></html>"###,
        ver = crate::app::APP_VERSION,
        rnd = app.process_id
    );

    HttpOkResult::Content {
        content_type: Some(crate::http::web_content_type::WebContentType::Html),
        content: content.into_bytes(),
    }
}
