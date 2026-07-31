use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult};

use crate::app::{AppContext, DbNamespace};

/// Header naming the namespace a request works in. No header — or an empty one —
/// means the default namespace, which is what every pre-namespace client sends.
///
/// The value is resolved HERE, from the request, for every action — one path,
/// so the query-parameter fallback below applies everywhere. The input
/// contracts additionally declare the header with `#[http_header(name = "ns")]`,
/// but only so that it shows up in the generated OpenAPI schema: swagger is
/// built from the contracts, and a header read straight off the request is
/// invisible to it. Those declared fields are documentation, not the source of
/// truth — reading them instead would quietly skip the query-parameter
/// fallback.
pub const NAMESPACE_HEADER: &str = "ns";

/// Namespace of an incoming HTTP request.
///
/// The namespace is created if this is the first time it is mentioned, the same
/// way a table is: `ns` is a client-owned name, not something an admin has to
/// register first.
pub async fn get_request_namespace(
    app: &Arc<AppContext>,
    ctx: &HttpContext,
) -> Result<Arc<DbNamespace>, HttpFailResult> {
    let namespace = match get_namespace_header(ctx) {
        Some(namespace) => Some(namespace),
        None => get_namespace_query_param(ctx),
    };

    let result = app.get_or_create_namespace(namespace).await?;

    Ok(result)
}

/// Namespace of a request which must not create one — the delete operations.
/// An unknown namespace is answered with "namespace not found" straight away
/// instead of being conjured into existence just so the delete can fail inside
/// it.
pub async fn get_request_namespace_existing(
    app: &Arc<AppContext>,
    ctx: &HttpContext,
) -> Result<Arc<DbNamespace>, HttpFailResult> {
    let namespace = match get_namespace_header(ctx) {
        Some(namespace) => Some(namespace),
        None => get_namespace_query_param(ctx),
    };

    let result = app.get_existing_namespace(namespace)?;

    Ok(result)
}

pub fn get_namespace_header(ctx: &HttpContext) -> Option<&str> {
    use my_http_server::HttpRequestHeaders;

    ctx.request
        .get_headers()
        .try_get_case_insensitive_as_str(NAMESPACE_HEADER)
        .ok()
        .flatten()
        .filter(|value| !value.is_empty())
}

/// Fallback for requests which can not carry a header at all: a browser
/// download is an `<a href>`, so the UI has no way to attach `ns` to it. The
/// header wins whenever both are present.
fn get_namespace_query_param(ctx: &HttpContext) -> Option<&str> {
    let query = ctx.request.get_uri().query()?;

    for pair in query.split('&') {
        // A valueless element (`?flag`) is somebody else's parameter, not the
        // end of the query string — keep looking.
        let (key, value) = match pair.split_once('=') {
            Some(result) => result,
            None => continue,
        };

        if key.eq_ignore_ascii_case(NAMESPACE_HEADER) {
            if value.is_empty() {
                return None;
            }

            return Some(value);
        }
    }

    None
}
