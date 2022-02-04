use std::{net::SocketAddr, sync::Arc};

use my_app_insights::AppInsightsTelemetry;
use my_http_server::{middlewares::StaticFilesMiddleware, MyHttpServer};
use my_http_server_app_insights::AppInsightsMiddleware;
use my_http_server_controllers::swagger::SwaggerMiddleware;

use crate::app::AppContext;

pub fn setup_server(
    app: Arc<AppContext>,
    app_insights_telemetry: Arc<AppInsightsTelemetry>,
    azure_connection: Option<Arc<my_azure_storage_sdk::AzureStorageConnection>>,
) {
    let mut http_server = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 5123)));

    let controllers = Arc::new(crate::http::controllers::builder::build(
        app.clone(),
        azure_connection.clone(),
    ));

    let app_insights_middleware = AppInsightsMiddleware::new(app_insights_telemetry.clone());
    http_server.add_middleware(Arc::new(app_insights_middleware));

    let swagger_middleware = SwaggerMiddleware::new(
        controllers.clone(),
        "MyNoSqlServer".to_string(),
        crate::app::APP_VERSION.to_string(),
    );

    http_server.add_middleware(Arc::new(swagger_middleware));
    http_server.add_middleware(controllers);

    http_server.add_middleware(Arc::new(StaticFilesMiddleware::new(None)));
    http_server.start(app.clone());
}
