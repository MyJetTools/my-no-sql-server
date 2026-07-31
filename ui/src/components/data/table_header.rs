use dioxus::prelude::*;

use crate::components::atoms::{Icon, IconKind};
use crate::models::TableApiModel;
use crate::utils::{format_bytes, format_unix_microseconds};

#[component]
pub fn TableHeader(
    name: String,
    stats: Option<TableApiModel>,
    on_refresh: EventHandler<()>,
) -> Element {
    let meta = if let Some(t) = stats {
        let size = format_bytes(t.data_size as f64);
        let persist_period = t
            .next_persist_time
            .filter(|v| *v > 0)
            .map(format_unix_microseconds)
            .unwrap_or_else(|| "—".to_string());
        let last_update = format_unix_microseconds(t.last_update_time);
        rsx! {
            div { class: "table-header__meta-item",
                "rows: " b { "{t.records_amount}" }
            }
            div { class: "table-header__meta-item",
                "partitions: " b { "{t.partitions_count}" }
            }
            div { class: "table-header__meta-item",
                "size: " b { "{size}" }
            }
            div { class: "table-header__meta-item",
                "next persist: " b { "{persist_period}" }
            }
            div { class: "table-header__meta-item",
                "updated: " b { "{last_update}" }
            }
        }
    } else {
        rsx! {
            div { class: "table-header__meta-item muted", "loading…" }
        }
    };

    rsx! {
        div { class: "table-header",
            span { class: "table-header__title", "{name}" }
            div { class: "table-header__meta", {meta} }
            div { class: "table-header__actions",
                button {
                    class: "topbar__icon-btn",
                    title: "Refresh",
                    onclick: move |_| on_refresh.call(()),
                    Icon { kind: IconKind::RefreshCw }
                }
                button { class: "topbar__icon-btn",
                    Icon { kind: IconKind::MoreHorizontal }
                }
            }
        }
    }
}
