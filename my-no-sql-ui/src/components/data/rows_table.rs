use dioxus::prelude::*;
use serde_json::Value;

use super::cell::{cell_class, cell_string};

pub const PARTITION_KEY: &str = "PartitionKey";
pub const ROW_KEY: &str = "RowKey";
pub const TIME_STAMP: &str = "TimeStamp";

#[component]
pub fn RowsTable(
    headers: Vec<String>,
    rows: Vec<Value>,
    selected_row_key: Option<String>,
    on_row_click: EventHandler<Value>,
) -> Element {
    if rows.is_empty() {
        return rsx! {
            div { class: "rows-wrap",
                div { class: "empty-state",
                    div { class: "empty-state__title", "No rows" }
                    div { class: "empty-state__sub", "This partition is empty." }
                }
            }
        };
    }

    let header_cells = headers.iter().map(|h| {
        let is_numeric = matches!(h.as_str(), "TimeStamp");
        let cls = if is_numeric { "num" } else { "" };
        rsx! { th { class: cls, "{h}" } }
    });

    let body_rows = rows.iter().enumerate().map(|(i, row)| {
        let rk = row
            .get(ROW_KEY)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let is_selected = selected_row_key.as_ref() == Some(&rk);

        let cells = headers.iter().map(|h| {
            let v = row.get(h.as_str()).cloned().unwrap_or(Value::Null);
            let mut cls = cell_class(h, &v).to_string();
            let mut is_num = false;
            if matches!(&v, Value::Number(_)) {
                is_num = true;
            }
            if h == PARTITION_KEY {
                cls = "pk".to_string();
            } else if h == ROW_KEY {
                cls = "rk".to_string();
            }
            if is_num {
                cls.push_str(" num");
            }
            let s = cell_string(&v);
            rsx! {
                td { class: "{cls}", title: "{s}", "{s}" }
            }
        });

        let row_val = row.clone();
        let row_cls = if is_selected { "selected" } else { "" };

        rsx! {
            tr {
                key: "{i}",
                class: row_cls,
                onclick: move |_| on_row_click.call(row_val.clone()),
                {cells}
            }
        }
    });

    rsx! {
        div { class: "rows-wrap",
            table { class: "rt",
                thead { tr { {header_cells} } }
                tbody { {body_rows} }
            }
        }
    }
}
