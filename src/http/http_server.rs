use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

use std::{net::SocketAddr, time::Duration};

use crate::app::AppContext;

use my_app_insights::AppInsightsTelemetry;
use std::sync::Arc;

pub async fn start(
    app: Arc<AppContext>,
    telemetry_writer: Arc<AppInsightsTelemetry>,
    addr: SocketAddr,
) {
    let sleep_duration = Duration::from_secs(1);

    while !app.states.is_initialized() {
        tokio::time::sleep(sleep_duration).await
    }

    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::System,
            format!("Http socket is started"),
            format!("Http socket: {}", addr),
        )
        .await;

    let make_service = make_service_fn(move |_| {
        let app = app.clone();
        let telemetry_writer = telemetry_writer.clone();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |_req| {
                handle_requests(_req, app.clone(), telemetry_writer.clone())
            }))
        }
    });

    Server::bind(&addr).serve(make_service).await.unwrap();
}

pub async fn handle_requests(
    req: Request<Body>,
    app: Arc<AppContext>,
    telemetry_writer: Arc<AppInsightsTelemetry>,
) -> hyper::Result<Response<Body>> {
    let response = super::router::route_requests(req, app, telemetry_writer).await;

    let response = match response {
        Ok(ok_result) => ok_result.into(),
        Err(fail_result) => fail_result.into(),
    };

    return Ok(response);
}
