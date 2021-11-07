use hyper::{Body, Method, Request};
use my_http_utils::{HttpFailResult, WebContentType};

use crate::app::AppContext;
use crate::http::http_ctx::HttpContext;
use std::sync::Arc;

use super::{
    controllers::{api, bulk, gc, logs, metrics, row, rows, status, tables, transactions},
    http_ok::HttpOkResult,
    static_files, swagger,
};

pub async fn route_requests(
    req: Request<Body>,
    app: Arc<AppContext>,
) -> Result<HttpOkResult, HttpFailResult> {
    if app.states.is_shutting_down() {
        return Err(HttpFailResult {
            content_type: WebContentType::Text,
            content: "App is being shutting down".to_string().into_bytes(),
            status_code: 301,
        });
    }

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
            return tables::list_of_tables::get(app.as_ref()).await;
        }

        (&Method::POST, "/tables/create") => {
            return tables::create::post(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/tables/createifnotexists") => {
            return tables::create_if_not_exists::post(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::PUT, "/tables/clean") => {
            return tables::clean::put(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::DELETE, "/tables/delete") => {
            return tables::delete::delete(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/tables/updatepersist") => {
            return tables::update_persist::post(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::GET, "/tables/partitionscount") => {
            return tables::partitions_count::get(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/tables/migratefrom") => {
            return tables::migrate_from::post(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::GET, "/row") => {
            return row::get(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::DELETE, "/row") => {
            return row::delete(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::PUT, "/row/replace") => {
            return row::replace::put(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/row/insert") => {
            return row::insert::post(HttpContext::new(req), app.as_ref()).await;
        }

        (&Method::POST, "/row/insertorreplace") => {
            return row::insert_or_replace::post(HttpContext::new(req), app.as_ref()).await;
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

        (&Method::POST, "/garbagecollector/gc") => {
            return gc::clean_and_keep_max_partitions_amount(HttpContext::new(req), app.as_ref())
                .await;
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
        return swagger::handle_request(path.as_str(), HttpContext::new(req)).await;
    }

    if path.starts_with("/css") {
        return static_files::serve_path(path.as_str()).await;
    }

    if path.starts_with("/js") {
        return static_files::serve_path(path.as_str()).await;
    }

    return Err(HttpFailResult::as_not_found("Not Found".to_string()));
}

fn get_index_page_content(app: &AppContext) -> HttpOkResult {
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
        content_type: Some(my_http_utils::WebContentType::Html),
        content: content.into_bytes(),
    }
}
