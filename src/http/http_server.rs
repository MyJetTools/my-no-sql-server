use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};

use std::net::SocketAddr;

use crate::app::AppServices;
use std::sync::Arc;

pub async fn start(app: Arc<AppServices>) {
    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::System,
            "Starting http server".to_string(),
            "*.5123".to_string(),
        )
        .await;

    let make_service = make_service_fn(move |_| {
        let app = app.clone();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |_req| {
                super::controllers::router::route_requests(_req, app.clone())
            }))
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 5123));

    Server::bind(&addr).serve(make_service).await.unwrap();
}
