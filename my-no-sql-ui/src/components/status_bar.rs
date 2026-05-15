use dioxus::prelude::*;

use crate::models::StatusBarApiModel;

#[component]
pub fn ConnectedStatusBar(data: StatusBarApiModel) -> Element {
    let compressed = if data.location.compress { "yes" } else { "no" };
    rsx! {
        div { class: "status-bar",
            div {
                "Connected: "
                b { style: "color:green", "online" }
            }
            div { "Location: {data.location.id}" }
            div { "Compressed: {compressed}" }
            div { "Tables: {data.tables_amount}" }
            div { "Tcp: {data.tcp_connections}" }
            div { "Http: {data.http_connections}/{data.used_http_connections}" }
            div { "Persist Q: {data.persist_amount}" }
            div { "Sync Q: {data.sync_queue_size}" }
        }
    }
}

#[component]
pub fn DisconnectedStatusBar() -> Element {
    rsx! {
        div { class: "status-bar",
            div {
                "Connected: "
                b { style: "color:red", "offline" }
            }
        }
    }
}
