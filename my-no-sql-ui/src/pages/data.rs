use dioxus::prelude::*;
use serde_json::Value;

use crate::api::{delete_row, get_partitions, get_rows, get_tables_list};
use crate::models::TableListItemApiModel;

const PARTITION_KEY: &str = "PartitionKey";
const ROW_KEY: &str = "RowKey";
const TIME_STAMP: &str = "TimeStamp";

#[derive(Clone, Default)]
struct RowsState {
    headers: Vec<String>,
    rows: Vec<Value>,
}

#[derive(Clone)]
struct DeleteDialog {
    partition_key: String,
    row_key: String,
}

#[component]
pub fn Data() -> Element {
    let mut tables = use_signal(Vec::<TableListItemApiModel>::new);
    let mut selected_table = use_signal(String::new);
    let mut partitions = use_signal(|| None::<Vec<String>>);
    let mut selected_partition = use_signal(|| None::<String>);
    let mut rows_state = use_signal(RowsState::default);
    let mut dialog = use_signal(|| None::<DeleteDialog>);
    let mut started = use_signal(|| false);

    let started_val = *started.read();
    let on_mount = move |_| {
        if started_val {
            return;
        }
        *started.write() = true;
        spawn(async move {
            if let Ok(list) = get_tables_list().await {
                let mut sorted = list;
                sorted.sort_by(|a, b| a.name.cmp(&b.name));
                tables.set(sorted);
            }
        });
    };

    let mut table_click = move |name: String| {
        selected_table.set(name.clone());
        selected_partition.set(None);
        partitions.set(None);
        rows_state.set(RowsState::default());

        spawn(async move {
            match get_partitions(&name).await {
                Ok(p) => {
                    let data = p.data;
                    if data.len() == 1 {
                        let pk = data[0].clone();
                        selected_partition.set(Some(pk.clone()));
                        if let Ok(rows) = get_rows(&name, &pk).await {
                            rows_state.set(build_rows_state(rows));
                        }
                    }
                    partitions.set(Some(data));
                }
                Err(err) => {
                    dioxus_utils::console_log(&format!("Partitions error: {}", err));
                }
            }
        });
    };

    let mut partition_click = move |pk: String| {
        selected_partition.set(Some(pk.clone()));
        rows_state.set(RowsState::default());
        let table_name = selected_table.read().clone();
        spawn(async move {
            if let Ok(rows) = get_rows(&table_name, &pk).await {
                rows_state.set(build_rows_state(rows));
            }
        });
    };

    let confirm_delete = move |_| {
        let Some(d) = dialog.read().clone() else {
            return;
        };
        let table_name = selected_table.read().clone();
        let pk = d.partition_key.clone();
        spawn(async move {
            if let Err(err) = delete_row(&table_name, &d.partition_key, &d.row_key).await {
                dioxus_utils::console_log(&format!("Delete error: {}", err));
                return;
            }
            rows_state.set(RowsState::default());
            if let Ok(rows) = get_rows(&table_name, &pk).await {
                rows_state.set(build_rows_state(rows));
            }
            dialog.set(None);
        });
    };

    let selected_table_val = selected_table.read().clone();
    let selected_partition_val = selected_partition.read().clone();
    let tables_list = tables.read().clone();
    let partitions_val = partitions.read().clone();
    let rows_val = rows_state.read().clone();
    let dialog_val = dialog.read().clone();

    let tables_render = tables_list.iter().map(|t| {
        let is_selected = t.name == selected_table_val;
        let bg = if is_selected { "lightgreen" } else { "white" };
        let name = t.name.clone();
        rsx! {
            div {
                class: "table-item",
                style: "background: {bg}",
                onclick: move |_| table_click(name.clone()),
                "{t.name}"
            }
        }
    });

    let table_data_content = if selected_table_val.is_empty() {
        rsx! {
            div { style: "padding:10px;",
                h1 { "Please select table" }
            }
        }
    } else if selected_partition_val.is_none() {
        let body = match &partitions_val {
            Some(p) if !p.is_empty() => {
                let badges = p.clone().into_iter().map(|pk| {
                    let pk_clone = pk.clone();
                    rsx! {
                        span {
                            class: "badge text-bg-secondary",
                            style: "margin-right:5px; cursor: pointer",
                            onclick: move |_| partition_click(pk_clone.clone()),
                            "{pk}"
                        }
                    }
                });
                rsx! {
                    div { style: "padding:10px",
                        h2 { "Please select partition" }
                        {badges}
                    }
                }
            }
            Some(_) => rsx! {
                h2 { "No records in this table" }
            },
            None => rsx! {
                div { style: "padding:10px;", "Loading partitions..." }
            },
        };
        body
    } else {
        let headers = rows_val.headers.clone();
        let header_cells = headers.iter().map(|h| {
            rsx! {
                th { "{h}" }
            }
        });

        let rows_iter = rows_val.rows.iter().enumerate().map(|(i, row)| {
            let pk = row
                .get(PARTITION_KEY)
                .and_then(|v| value_as_string(v))
                .unwrap_or_default();
            let rk = row
                .get(ROW_KEY)
                .and_then(|v| value_as_string(v))
                .unwrap_or_default();

            let cells = headers.iter().map(|header| {
                let value = row
                    .get(header.as_str())
                    .map(|v| value_as_string(v).unwrap_or_else(|| v.to_string()))
                    .unwrap_or_default();
                rsx! {
                    td {
                        div { class: "table-row", "{value}" }
                    }
                }
            });

            let pk_for_click = pk.clone();
            let rk_for_click = rk.clone();
            rsx! {
                tr { key: "{i}",
                    th { style: "width:18px",
                        img {
                            src: "/ico/delete.svg",
                            style: "height:16px;cursor: pointer",
                            onclick: move |_| {
                                dialog
                                    .set(
                                        Some(DeleteDialog {
                                            partition_key: pk_for_click.clone(),
                                            row_key: rk_for_click.clone(),
                                        }),
                                    );
                            },
                        }
                    }
                    {cells}
                }
            }
        });

        rsx! {
            button {
                class: "btn btn-success",
                style: "margin: 5px;",
                onclick: move |_| {
                    selected_partition.set(None);
                },
                "Select partition"
            }
            table { class: "table table-striped",
                thead {
                    tr {
                        th {}
                        {header_cells}
                    }
                }
                tbody { {rows_iter} }
            }
        }
    };

    let dialog_render = if let Some(d) = dialog_val {
        let pk = d.partition_key.clone();
        let rk = d.row_key.clone();
        rsx! {
            div { class: "dialog-pad",
                div { class: "dialog-window",
                    div { class: "dialog-header",
                        h4 { style: "margin: 0; padding:0;", "Please confirm you want to delete entity" }
                    }
                    div { class: "dialog-content",
                        div {
                            div {
                                b { "Please confirm that you want to delete that record" }
                            }
                            div {
                                "PartitionKey: "
                                b { "{pk}" }
                            }
                            div {
                                "RowKey: "
                                b { "{rk}" }
                            }
                        }
                    }
                    div { class: "dialog-footer",
                        div { class: "btn-group", style: "box-shadow: 0 0 6px #0000002e;",
                            button {
                                style: "padding: 5px 15px;",
                                class: "btn btn-sm btn-warning",
                                onclick: confirm_delete,
                                "Delete"
                            }
                            button {
                                style: "padding: 5px 15px;",
                                class: "btn btn-sm btn-outline-dark",
                                onclick: move |_| dialog.set(None),
                                "Cancel"
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    };

    rsx! {
        section { onmounted: on_mount,
            div { class: "tables", {tables_render} }
            div { class: "table-data", {table_data_content} }
            {dialog_render}
        }
    }
}

fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) => Some(s.clone()),
        Value::Array(_) | Value::Object(_) => Some(value.to_string()),
    }
}

fn build_rows_state(rows: Vec<Value>) -> RowsState {
    let mut headers: Vec<String> = vec![
        PARTITION_KEY.to_string(),
        ROW_KEY.to_string(),
        TIME_STAMP.to_string(),
    ];

    for row in rows.iter() {
        if let Value::Object(map) = row {
            for key in map.keys() {
                if key == PARTITION_KEY || key == ROW_KEY || key == TIME_STAMP {
                    continue;
                }
                if !headers.iter().any(|h| h == key) {
                    headers.push(key.clone());
                }
            }
        }
    }

    RowsState { headers, rows }
}
