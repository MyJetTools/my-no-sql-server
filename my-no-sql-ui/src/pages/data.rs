use std::collections::HashSet;

use dioxus::prelude::*;
use serde_json::Value;

use crate::api::{bulk_delete_rows, delete_row, get_partitions, get_rows, get_tables_list};
use crate::models::TableListItemApiModel;
use crate::storage;

const PARTITION_KEY: &str = "PartitionKey";
const ROW_KEY: &str = "RowKey";
const TIME_STAMP: &str = "TimeStamp";

#[derive(Clone, Default)]
struct RowsState {
    headers: Vec<String>,
    rows: Vec<Value>,
}

#[derive(Clone)]
enum DialogState {
    DeleteOne {
        partition_key: String,
        row_key: String,
    },
    DeleteMany {
        partition_key: String,
        row_keys: Vec<String>,
    },
}

#[component]
pub fn Data() -> Element {
    let mut tables = use_signal(Vec::<TableListItemApiModel>::new);
    let mut selected_table = use_signal(String::new);
    let mut partitions = use_signal(|| None::<Vec<String>>);
    let mut selected_partition = use_signal(|| None::<String>);
    let mut rows_state = use_signal(RowsState::default);
    let mut selected_rows = use_signal(HashSet::<String>::new);
    let mut dialog = use_signal(|| None::<DialogState>);
    let mut started = use_signal(|| false);

    let started_val = *started.read();
    let on_mount = move |_| {
        if started_val {
            return;
        }
        *started.write() = true;
        let saved_table = storage::load_selected_table();
        let saved_partition = storage::load_selected_partition();
        spawn(async move {
            let list = match get_tables_list().await {
                Ok(l) => l,
                Err(err) => {
                    dioxus_utils::console_log(&format!("Tables error: {}", err));
                    return;
                }
            };
            let mut sorted = list;
            sorted.sort_by(|a, b| a.name.cmp(&b.name));
            tables.set(sorted.clone());

            let Some(table_name) = saved_table
                .filter(|name| sorted.iter().any(|t| &t.name == name))
            else {
                return;
            };
            selected_table.set(table_name.clone());

            let parts = match get_partitions(&table_name).await {
                Ok(p) => p.data,
                Err(err) => {
                    dioxus_utils::console_log(&format!("Partitions error: {}", err));
                    return;
                }
            };

            let restored_pk = saved_partition
                .filter(|pk| parts.contains(pk))
                .or_else(|| {
                    if parts.len() == 1 {
                        Some(parts[0].clone())
                    } else {
                        None
                    }
                });

            partitions.set(Some(parts));

            if let Some(pk) = restored_pk {
                selected_partition.set(Some(pk.clone()));
                storage::save_selected_partition(Some(&pk));
                if let Ok(rows) = get_rows(&table_name, &pk).await {
                    rows_state.set(build_rows_state(rows));
                }
            }
        });
    };

    let mut table_click = move |name: String| {
        selected_table.set(name.clone());
        selected_partition.set(None);
        partitions.set(None);
        rows_state.set(RowsState::default());
        selected_rows.write().clear();
        storage::save_selected_table(&name);
        storage::save_selected_partition(None);

        spawn(async move {
            match get_partitions(&name).await {
                Ok(p) => {
                    let data = p.data;
                    if data.len() == 1 {
                        let pk = data[0].clone();
                        selected_partition.set(Some(pk.clone()));
                        storage::save_selected_partition(Some(&pk));
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
        selected_rows.write().clear();
        storage::save_selected_partition(Some(&pk));
        let table_name = selected_table.read().clone();
        spawn(async move {
            if let Ok(rows) = get_rows(&table_name, &pk).await {
                rows_state.set(build_rows_state(rows));
            }
        });
    };

    let confirm_action = move |_| {
        let Some(d) = dialog.read().clone() else {
            return;
        };
        let table_name = selected_table.read().clone();

        match d {
            DialogState::DeleteOne {
                partition_key,
                row_key,
            } => {
                spawn(async move {
                    if let Err(err) =
                        delete_row(&table_name, &partition_key, &row_key).await
                    {
                        dioxus_utils::console_log(&format!("Delete error: {}", err));
                        return;
                    }
                    refresh_after_delete(
                        rows_state,
                        selected_rows,
                        dialog,
                        &table_name,
                        &partition_key,
                    )
                    .await;
                });
            }
            DialogState::DeleteMany {
                partition_key,
                row_keys,
            } => {
                spawn(async move {
                    if let Err(err) =
                        bulk_delete_rows(&table_name, &partition_key, &row_keys).await
                    {
                        dioxus_utils::console_log(&format!("Bulk delete error: {}", err));
                        return;
                    }
                    refresh_after_delete(
                        rows_state,
                        selected_rows,
                        dialog,
                        &table_name,
                        &partition_key,
                    )
                    .await;
                });
            }
        }
    };

    let selected_table_val = selected_table.read().clone();
    let selected_partition_val = selected_partition.read().clone();
    let tables_list = tables.read().clone();
    let partitions_val = partitions.read().clone();
    let rows_val = rows_state.read().clone();
    let selected_rows_val = selected_rows.read().clone();
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
        let partition_key_val = selected_partition_val.clone().unwrap_or_default();
        let headers = rows_val.headers.clone();
        let total_rows = rows_val.rows.len();
        let selected_count = selected_rows_val.len();
        let all_selected = total_rows > 0 && selected_count == total_rows;

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

            let is_checked = selected_rows_val.contains(&rk);

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
            let rk_for_check = rk.clone();
            rsx! {
                tr { key: "{i}",
                    th { style: "width:24px; text-align:center;",
                        input {
                            r#type: "checkbox",
                            checked: is_checked,
                            onclick: move |evt| {
                                evt.stop_propagation();
                            },
                            onchange: move |_| {
                                let mut w = selected_rows.write();
                                if w.contains(&rk_for_check) {
                                    w.remove(&rk_for_check);
                                } else {
                                    w.insert(rk_for_check.clone());
                                }
                            },
                        }
                    }
                    th { style: "width:18px",
                        img {
                            src: "/ico/delete.svg",
                            style: "height:16px;cursor: pointer",
                            onclick: move |_| {
                                dialog
                                    .set(
                                        Some(DialogState::DeleteOne {
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

        let pk_for_select_all = partition_key_val.clone();
        let rows_for_select_all = rows_val.rows.clone();

        let pk_for_bulk = partition_key_val.clone();
        let selected_for_bulk = selected_rows_val.clone();
        let bulk_button = if selected_count > 0 {
            rsx! {
                button {
                    class: "btn btn-danger",
                    style: "margin: 5px;",
                    onclick: move |_| {
                        let row_keys: Vec<String> = selected_for_bulk.iter().cloned().collect();
                        dialog
                            .set(
                                Some(DialogState::DeleteMany {
                                    partition_key: pk_for_bulk.clone(),
                                    row_keys,
                                }),
                            );
                    },
                    "Bulk-delete ({selected_count})"
                }
            }
        } else {
            rsx! {}
        };

        rsx! {
            div { class: "table-toolbar",
                button {
                    class: "btn btn-success",
                    style: "margin: 5px;",
                    onclick: move |_| {
                        selected_partition.set(None);
                        selected_rows.write().clear();
                        storage::save_selected_partition(None);
                    },
                    "Select partition"
                }
                {bulk_button}
            }
            table { class: "table table-striped",
                thead {
                    tr {
                        th { style: "width:24px; text-align:center;",
                            input {
                                r#type: "checkbox",
                                checked: all_selected,
                                onchange: move |_| {
                                    let mut w = selected_rows.write();
                                    if all_selected {
                                        w.clear();
                                    } else {
                                        w.clear();
                                        for row in rows_for_select_all.iter() {
                                            if let Some(rk) = row
                                                .get(ROW_KEY)
                                                .and_then(|v| value_as_string(v))
                                            {
                                                w.insert(rk);
                                            }
                                        }
                                    }
                                    let _ = &pk_for_select_all;
                                },
                            }
                        }
                        th {}
                        {header_cells}
                    }
                }
                tbody { {rows_iter} }
            }
        }
    };

    let dialog_render = match dialog_val {
        Some(DialogState::DeleteOne {
            partition_key,
            row_key,
        }) => {
            let pk = partition_key.clone();
            let rk = row_key.clone();
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
                                    onclick: confirm_action,
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
        }
        Some(DialogState::DeleteMany {
            partition_key,
            row_keys,
        }) => {
            let pk = partition_key.clone();
            let count = row_keys.len();
            let preview = row_keys
                .iter()
                .take(10)
                .map(|rk| {
                    rsx! {
                        div { "{rk}" }
                    }
                });
            let extra = if row_keys.len() > 10 {
                let more = row_keys.len() - 10;
                rsx! {
                    div { i { "... +{more} more" } }
                }
            } else {
                rsx! {}
            };
            rsx! {
                div { class: "dialog-pad",
                    div { class: "dialog-window",
                        div { class: "dialog-header",
                            h4 { style: "margin: 0; padding:0;", "Please confirm bulk-delete" }
                        }
                        div { class: "dialog-content",
                            div {
                                div {
                                    b { "Delete {count} record(s)?" }
                                }
                                div {
                                    "PartitionKey: "
                                    b { "{pk}" }
                                }
                                div { style: "margin-top:8px; max-height:200px; overflow:auto; font-size:12px;",
                                    {preview}
                                    {extra}
                                }
                            }
                        }
                        div { class: "dialog-footer",
                            div { class: "btn-group", style: "box-shadow: 0 0 6px #0000002e;",
                                button {
                                    style: "padding: 5px 15px;",
                                    class: "btn btn-sm btn-warning",
                                    onclick: confirm_action,
                                    "Delete {count}"
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
        }
        None => rsx! {},
    };

    rsx! {
        section { onmounted: on_mount,
            div { class: "tables", {tables_render} }
            div { class: "table-data", {table_data_content} }
            {dialog_render}
        }
    }
}

async fn refresh_after_delete(
    mut rows_state: Signal<RowsState>,
    mut selected_rows: Signal<HashSet<String>>,
    mut dialog: Signal<Option<DialogState>>,
    table_name: &str,
    partition_key: &str,
) {
    rows_state.set(RowsState::default());
    selected_rows.write().clear();
    if let Ok(rows) = get_rows(table_name, partition_key).await {
        rows_state.set(build_rows_state(rows));
    }
    dialog.set(None);
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
