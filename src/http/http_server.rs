use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

use std::net::SocketAddr;

use crate::app::AppContext;
use std::sync::Arc;

pub async fn start(app: Arc<AppContext>, addr: SocketAddr) {
    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::System,
            format!("Starting http server {}", addr),
            format!("Starting http server {}", addr),
        )
        .await;

    let make_service = make_service_fn(move |_| {
        let app = app.clone();

        async move { Ok::<_, hyper::Error>(service_fn(move |_req| handle_requests(_req, app.clone()))) }
    });

    Server::bind(&addr).serve(make_service).await.unwrap();
}

pub async fn handle_requests(
    req: Request<Body>,
    app: Arc<AppContext>,
) -> hyper::Result<Response<Body>> {
    let response = super::router::route_requests(req, app).await;

    let response = match response {
        Ok(ok_result) => ok_result.into(),
        Err(fail_result) => fail_result.into(),
    };

    return Ok(response);
}
