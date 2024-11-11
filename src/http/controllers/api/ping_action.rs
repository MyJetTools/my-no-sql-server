use my_http_server::{macros::http_route, HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

#[http_route(
    method: "GET",
    route: "/api/Ping",
    controller: "Monitoring",
    description: "Endpoint to ping the service",
    summary: "Endpoint to ping the service",
    result:[
        {status_code: 204, description: "Ok result"},
    ]
)]
pub struct PingAction;

async fn handle_request(
    _: &PingAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    HttpOutput::Empty.into_ok_result(false).into()
}
