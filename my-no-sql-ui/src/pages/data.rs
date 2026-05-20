use std::collections::{BTreeMap, HashMap, HashSet};

use dioxus::prelude::*;
use serde_json::Value;

use crate::AppContext;
use crate::api::{
    bulk_delete_many, bulk_delete_rows, delete_row, get_partitions, get_rows, get_tables_list,
};
use crate::components::atoms::{Badge, BadgeTone, Icon, IconKind};
use crate::components::data::{
    PARTITION_KEY, PartitionsPane, ROW_KEY, RowDrawer, RowsTable, TableHeader, TableToolbar,
    TablesPane, TIME_STAMP,
};
use crate::models::{StatusApiModel, TableApiModel, TableListItemApiModel};
use crate::storage;

#[derive(Clone)]
enum DialogState {
    DeleteOne {
        partition_key: String,
        row_key: String,
    },
    BulkDelete {
        partition_key: String,
        row_keys: Vec<String>,
    },
    PasteDelete {
        raw: String,
        parsed: Option<BTreeMap<String, Vec<String>>>,
        total_rows: usize,
        partitions_touched: usize,
        error: Option<String>,
    },
}

#[derive(Default)]
struct DataState {
    tables: Vec<TableListItemApiModel>,
    selected_table: String,
    partitions: Option<Vec<String>>,
    selected_partition: Option<String>,
    headers: Vec<String>,
    rows: Vec<Value>,
    selected_row: Option<Value>,
    drawer_open: bool,
    started: bool,
    dialog: Option<DialogState>,
    checked_keys: HashSet<String>,
}

#[component]
pub fn Data() -> Element {
    let mut cs = use_signal(DataState::default);
    let row_filter = use_signal(String::new);
    let app_ctx = use_context::<Signal<AppContext>>();

    let cs_ra = cs.read();
    let started = cs_ra.started;
    drop(cs_ra);

    let on_mount = move |_| {
        if started {
            return;
        }
        cs.write().started = true;

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
            cs.write().tables = sorted.clone();

            let Some(table_name) = saved_table
                .filter(|name| sorted.iter().any(|t| &t.name == name))
            else {
                return;
            };
            cs.write().selected_table = table_name.clone();

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

            cs.write().partitions = Some(parts);

            if let Some(pk) = restored_pk {
                cs.write().selected_partition = Some(pk.clone());
                storage::save_selected_partition(Some(&pk));
                if let Ok(rows) = get_rows(&table_name, &pk).await {
                    let (headers, rows) = build_rows_state(rows);
                    let mut w = cs.write();
                    w.headers = headers;
                    w.rows = rows;
                }
            }
        });
    };

    let mut select_table = move |name: String| {
        {
            let mut w = cs.write();
            w.selected_table = name.clone();
            w.selected_partition = None;
            w.partitions = None;
            w.headers = Vec::new();
            w.rows = Vec::new();
            w.selected_row = None;
            w.drawer_open = false;
            w.checked_keys.clear();
        }
        storage::save_selected_table(&name);
        storage::save_selected_partition(None);

        spawn(async move {
            match get_partitions(&name).await {
                Ok(p) => {
                    let data = p.data;
                    if data.len() == 1 {
                        let pk = data[0].clone();
                        cs.write().selected_partition = Some(pk.clone());
                        storage::save_selected_partition(Some(&pk));
                        if let Ok(rows) = get_rows(&name, &pk).await {
                            let (headers, rows) = build_rows_state(rows);
                            let mut w = cs.write();
                            w.headers = headers;
                            w.rows = rows;
                        }
                    }
                    cs.write().partitions = Some(data);
                }
                Err(err) => {
                    dioxus_utils::console_log(&format!("Partitions error: {}", err));
                }
            }
        });
    };

    let mut select_partition = move |pk: String| {
        {
            let mut w = cs.write();
            w.selected_partition = Some(pk.clone());
            w.headers = Vec::new();
            w.rows = Vec::new();
            w.selected_row = None;
            w.drawer_open = false;
            w.checked_keys.clear();
        }
        storage::save_selected_partition(Some(&pk));
        let table_name = cs.read().selected_table.clone();
        spawn(async move {
            if let Ok(rows) = get_rows(&table_name, &pk).await {
                let (headers, rows) = build_rows_state(rows);
                let mut w = cs.write();
                w.headers = headers;
                w.rows = rows;
            }
        });
    };

    let on_row_click = move |row: Value| {
        let mut w = cs.write();
        w.selected_row = Some(row);
        w.drawer_open = true;
    };

    let close_drawer = move |_| {
        let mut w = cs.write();
        w.selected_row = None;
        w.drawer_open = false;
    };

    let confirm_delete = move |_| {
        let cs_ra = cs.read();
        let dialog_val = cs_ra.dialog.clone();
        let table_name = cs_ra.selected_table.clone();
        drop(cs_ra);
        match dialog_val {
            Some(DialogState::DeleteOne {
                partition_key,
                row_key,
            }) => {
                spawn(async move {
                    if let Err(err) = delete_row(&table_name, &partition_key, &row_key).await {
                        dioxus_utils::console_log(&format!("Delete error: {}", err));
                        return;
                    }
                    {
                        let mut w = cs.write();
                        w.dialog = None;
                        w.drawer_open = false;
                        w.selected_row = None;
                        w.checked_keys.remove(&row_key);
                    }
                    if let Ok(rows) = get_rows(&table_name, &partition_key).await {
                        let (headers, rows) = build_rows_state(rows);
                        let mut w = cs.write();
                        w.headers = headers;
                        w.rows = rows;
                    }
                });
            }
            Some(DialogState::BulkDelete {
                partition_key,
                row_keys,
            }) => {
                spawn(async move {
                    if let Err(err) =
                        bulk_delete_rows(&table_name, &partition_key, &row_keys).await
                    {
                        dioxus_utils::console_log(&format!("Bulk delete error: {}", err));
                        return;
                    }
                    {
                        let mut w = cs.write();
                        w.dialog = None;
                        w.drawer_open = false;
                        w.selected_row = None;
                        for rk in &row_keys {
                            w.checked_keys.remove(rk);
                        }
                    }
                    if let Ok(rows) = get_rows(&table_name, &partition_key).await {
                        let (headers, rows) = build_rows_state(rows);
                        let mut w = cs.write();
                        w.headers = headers;
                        w.rows = rows;
                    }
                });
            }
            Some(DialogState::PasteDelete {
                parsed: Some(grouped),
                ..
            }) => {
                spawn(async move {
                    if let Err(err) = bulk_delete_many(&table_name, &grouped).await {
                        dioxus_utils::console_log(&format!("Paste delete error: {}", err));
                        return;
                    }
                    let current_pk = {
                        let mut w = cs.write();
                        w.dialog = None;
                        w.drawer_open = false;
                        w.selected_row = None;
                        w.checked_keys.clear();
                        w.selected_partition.clone()
                    };
                    if let Some(pk) = current_pk {
                        if let Ok(rows) = get_rows(&table_name, &pk).await {
                            let (headers, rows) = build_rows_state(rows);
                            let mut w = cs.write();
                            w.headers = headers;
                            w.rows = rows;
                        }
                    }
                });
            }
            Some(DialogState::PasteDelete { parsed: None, .. }) => {}
            None => {}
        }
    };

    let toggle_row_check = move |rk: String| {
        let mut w = cs.write();
        if w.checked_keys.contains(&rk) {
            w.checked_keys.remove(&rk);
        } else {
            w.checked_keys.insert(rk);
        }
    };

    let toggle_all_check = move |check_all: bool| {
        let mut w = cs.write();
        if check_all {
            let keys: Vec<String> = w
                .rows
                .iter()
                .filter_map(|r| {
                    r.get(ROW_KEY)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                })
                .collect();
            for k in keys {
                w.checked_keys.insert(k);
            }
        } else {
            w.checked_keys.clear();
        }
    };

    // Re-read state for rendering
    let cs_ra = cs.read();
    let tables = cs_ra.tables.clone();
    let selected_table = cs_ra.selected_table.clone();
    let partitions = cs_ra.partitions.clone();
    let selected_partition = cs_ra.selected_partition.clone();
    let headers = cs_ra.headers.clone();
    let all_rows = cs_ra.rows.clone();
    let selected_row = cs_ra.selected_row.clone();
    let drawer_open = cs_ra.drawer_open;
    let dialog_val = cs_ra.dialog.clone();
    let checked_keys = cs_ra.checked_keys.clone();
    drop(cs_ra);

    // Derive writers per table for the selected table
    let ctx_ra = app_ctx.read();
    let status: Option<StatusApiModel> = ctx_ra.status.clone();
    drop(ctx_ra);

    let writer_tables: HashSet<String> = build_writer_tables(&status);
    let (writer_apps_for_selected, reader_count_for_selected) =
        derive_table_connectivity(&status, &selected_table);
    let table_stats = derive_table_stats(&status, &selected_table);
    let row_counts_by_table = derive_table_row_counts(&status);
    let partition_counts = HashMap::<String, usize>::new();

    // Filter rows
    let filter_str = row_filter.read().to_lowercase();
    let filtered_rows: Vec<Value> = if filter_str.is_empty() {
        all_rows.clone()
    } else {
        all_rows
            .iter()
            .filter(|row| row.to_string().to_lowercase().contains(&filter_str))
            .cloned()
            .collect()
    };

    let selected_row_key = selected_row.as_ref().and_then(|r| {
        r.get(ROW_KEY)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    });

    let center_content = if selected_table.is_empty() {
        render_empty_state(tables.clone(), move |name| select_table(name))
    } else {
        let on_refresh_table = {
            let name = selected_table.clone();
            move |_| {
                let table = name.clone();
                spawn(async move {
                    let pk_opt = cs.read().selected_partition.clone();
                    if let Some(pk) = pk_opt {
                        if let Ok(rows) = get_rows(&table, &pk).await {
                            let (headers, rows) = build_rows_state(rows);
                            let mut w = cs.write();
                            w.headers = headers;
                            w.rows = rows;
                        }
                    }
                });
            }
        };
        let on_export_click = {
            let table = selected_table.clone();
            let pk_opt = selected_partition.clone();
            move |_| {
                let Some(pk) = pk_opt.clone() else { return };
                let url = crate::api::download_rows_url(&table, &pk);
                let script = format!(
                    "window.location.href = {};",
                    serde_json::to_string(&url).unwrap_or_else(|_| "\"\"".to_string())
                );
                let _ = dioxus::document::eval(&script);
            }
        };
        let export_enabled = selected_partition.is_some();
        let checked_in_partition: Vec<String> = filtered_rows
            .iter()
            .filter_map(|r| {
                r.get(ROW_KEY)
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            })
            .filter(|k| checked_keys.contains(k))
            .collect();
        let checked_count = checked_in_partition.len();

        let bulk_bar = if checked_count > 0 {
            let pk_opt = selected_partition.clone();
            let keys_for_delete = checked_in_partition.clone();
            rsx! {
                div { class: "bulk-bar",
                    span { class: "bulk-bar__count", "{checked_count} selected" }
                    div { class: "bulk-bar__spacer" }
                    button {
                        class: "btn btn--ghost btn--sm",
                        onclick: move |_| { cs.write().checked_keys.clear(); },
                        "Clear"
                    }
                    button {
                        class: "btn btn--danger btn--sm",
                        onclick: move |_| {
                            let Some(pk) = pk_opt.clone() else { return };
                            cs.write().dialog = Some(DialogState::BulkDelete {
                                partition_key: pk,
                                row_keys: keys_for_delete.clone(),
                            });
                        },
                        "Delete selected"
                    }
                }
            }
        } else {
            rsx! {}
        };

        rsx! {
            div { class: "rows-col",
                TableHeader {
                    name: selected_table.clone(),
                    stats: table_stats.clone(),
                    on_refresh: on_refresh_table,
                }
                TableToolbar {
                    filter_value: row_filter,
                    writer_tags: writer_apps_for_selected.clone(),
                    reader_count: reader_count_for_selected,
                    on_export: on_export_click,
                    export_enabled,
                    on_paste_delete: move |_| {
                        cs.write().dialog = Some(DialogState::PasteDelete {
                            raw: String::new(),
                            parsed: None,
                            total_rows: 0,
                            partitions_touched: 0,
                            error: None,
                        });
                    },
                    paste_enabled: true,
                }
                {bulk_bar}
                RowsTable {
                    headers: headers.clone(),
                    rows: filtered_rows,
                    selected_row_key,
                    on_row_click,
                    selectable: true,
                    checked_keys: checked_keys.clone(),
                    on_toggle_row: toggle_row_check,
                    on_toggle_all: toggle_all_check,
                }
            }
        }
    };

    let partitions_content = if selected_table.is_empty() {
        rsx! { aside { class: "partitions-pane" } }
    } else {
        let parts = partitions.clone().unwrap_or_default();
        rsx! {
            PartitionsPane {
                partitions: parts,
                counts: partition_counts,
                selected: selected_partition.clone(),
                on_select: move |pk| select_partition(pk),
            }
        }
    };

    let drawer_content = if drawer_open && selected_row.is_some() {
        let row = selected_row.clone().unwrap();
        let pk_val = row
            .get(PARTITION_KEY)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();
        let rk_val = row
            .get(ROW_KEY)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();
        rsx! {
            RowDrawer {
                row,
                on_close: close_drawer,
                on_delete: move |_| {
                    cs.write().dialog = Some(DialogState::DeleteOne {
                        partition_key: pk_val.clone(),
                        row_key: rk_val.clone(),
                    });
                },
            }
        }
    } else {
        rsx! {}
    };

    let dialog_render = match dialog_val {
        Some(DialogState::DeleteOne { partition_key, row_key }) => rsx! {
            div { class: "dialog-overlay",
                div { class: "dialog",
                    div { class: "dialog__header", "Confirm delete" }
                    div { class: "dialog__body",
                        "Delete row "
                        b { "{row_key}" }
                        " from partition "
                        b { "{partition_key}" }
                        "?"
                    }
                    div { class: "dialog__footer",
                        button {
                            class: "btn btn--ghost btn--sm",
                            onclick: move |_| { cs.write().dialog = None; },
                            "Cancel"
                        }
                        button { class: "btn btn--danger btn--sm", onclick: confirm_delete,
                            "Delete"
                        }
                    }
                }
            }
        },
        Some(DialogState::BulkDelete { partition_key, row_keys }) => {
            let total = row_keys.len();
            const PREVIEW_LIMIT: usize = 50;
            let preview: Vec<String> = row_keys.iter().take(PREVIEW_LIMIT).cloned().collect();
            let extra = total.saturating_sub(preview.len());
            let items = preview.into_iter().map(|k| rsx! {
                div { class: "dialog__list-item", "{k}" }
            });
            let extra_line = if extra > 0 {
                rsx! { div { class: "dialog__list-extra", "+ {extra} more" } }
            } else {
                rsx! {}
            };
            rsx! {
                div { class: "dialog-overlay",
                    div { class: "dialog",
                        div { class: "dialog__header", "Confirm bulk delete" }
                        div { class: "dialog__body",
                            "Delete "
                            b { "{total}" }
                            " row(s) from partition "
                            b { "{partition_key}" }
                            "?"
                            div { class: "dialog__list",
                                {items}
                                {extra_line}
                            }
                        }
                        div { class: "dialog__footer",
                            button {
                                class: "btn btn--ghost btn--sm",
                                onclick: move |_| { cs.write().dialog = None; },
                                "Cancel"
                            }
                            button { class: "btn btn--danger btn--sm", onclick: confirm_delete,
                                "Delete {total} row(s)"
                            }
                        }
                    }
                }
            }
        }
        Some(DialogState::PasteDelete {
            raw,
            parsed,
            total_rows,
            partitions_touched,
            error,
        }) => {
            let parse_ready = parsed.is_some();
            let total = total_rows;
            let partitions_n = partitions_touched;
            let error_text = error.clone();
            let target_table = selected_table.clone();

            let on_textarea_input = move |evt: dioxus::events::FormEvent| {
                let new_raw = evt.value();
                let mut w = cs.write();
                if let Some(DialogState::PasteDelete {
                    raw: r,
                    parsed: p,
                    total_rows: t,
                    partitions_touched: pt,
                    error: e,
                }) = w.dialog.as_mut()
                {
                    *r = new_raw;
                    *p = None;
                    *t = 0;
                    *pt = 0;
                    *e = None;
                }
            };

            let on_parse_click = move |_| {
                let raw_snapshot = {
                    let cs_ra = cs.read();
                    match cs_ra.dialog.as_ref() {
                        Some(DialogState::PasteDelete { raw, .. }) => raw.clone(),
                        _ => return,
                    }
                };
                match parse_paste_delete_input(&raw_snapshot) {
                    Ok((grouped, total, partitions_touched)) => {
                        let mut w = cs.write();
                        if let Some(DialogState::PasteDelete {
                            parsed,
                            total_rows,
                            partitions_touched: pt_field,
                            error,
                            ..
                        }) = w.dialog.as_mut()
                        {
                            *parsed = Some(grouped);
                            *total_rows = total;
                            *pt_field = partitions_touched;
                            *error = None;
                        }
                    }
                    Err(msg) => {
                        let mut w = cs.write();
                        if let Some(DialogState::PasteDelete {
                            parsed,
                            total_rows,
                            partitions_touched: pt_field,
                            error,
                            ..
                        }) = w.dialog.as_mut()
                        {
                            *parsed = None;
                            *total_rows = 0;
                            *pt_field = 0;
                            *error = Some(msg);
                        }
                    }
                }
            };

            let preview_items: Vec<(String, String)> = parsed
                .as_ref()
                .map(|g| {
                    const PREVIEW_LIMIT: usize = 50;
                    let mut out = Vec::new();
                    'outer: for (pk, rks) in g.iter() {
                        for rk in rks {
                            out.push((pk.clone(), rk.clone()));
                            if out.len() >= PREVIEW_LIMIT {
                                break 'outer;
                            }
                        }
                    }
                    out
                })
                .unwrap_or_default();
            let preview_shown = preview_items.len();
            let preview_extra = total.saturating_sub(preview_shown);
            let preview_items_render = preview_items.into_iter().map(|(pk, rk)| {
                rsx! {
                    div { class: "dialog__list-item", "pk={pk} / rk={rk}" }
                }
            });
            let preview_extra_line = if preview_extra > 0 {
                rsx! { div { class: "dialog__list-extra", "+ {preview_extra} more" } }
            } else {
                rsx! {}
            };

            let placeholder_text: &'static str = "[\n  { \"PartitionKey\": \"a\", \"RowKey\": \"1\" },\n  { \"PartitionKey\": \"b\", \"RowKey\": \"5\" }\n]";

            let error_render = if let Some(msg) = error_text {
                rsx! {
                    div { class: "dialog__error", "{msg}" }
                }
            } else {
                rsx! {}
            };

            let summary_render = if parse_ready {
                rsx! {
                    div { style: "margin: 6px 0;",
                        "Will delete "
                        b { "{total}" }
                        " row(s) across "
                        b { "{partitions_n}" }
                        " partition(s)"
                    }
                }
            } else {
                rsx! {}
            };

            rsx! {
                div { class: "dialog-overlay",
                    div { class: "dialog",
                        div { class: "dialog__header", "Paste & delete" }
                        div { class: "dialog__body",
                            div { style: "margin-bottom: 6px;",
                                "Target table: "
                                b { "{target_table}" }
                            }
                            div { style: "margin-bottom: 6px; font-size: 12px; color: gray;",
                                "Paste a JSON array of objects with "
                                code { "PartitionKey" }
                                " and "
                                code { "RowKey" }
                                " fields."
                            }
                            textarea {
                                value: "{raw}",
                                oninput: on_textarea_input,
                                rows: "10",
                                style: "width: 100%; font-family: monospace; font-size: 12px;",
                                placeholder: placeholder_text,
                            }
                            {error_render}
                            {summary_render}
                            div { class: "dialog__list",
                                {preview_items_render}
                                {preview_extra_line}
                            }
                        }
                        div { class: "dialog__footer",
                            button {
                                class: "btn btn--ghost btn--sm",
                                onclick: move |_| { cs.write().dialog = None; },
                                "Cancel"
                            }
                            button {
                                class: "btn btn--sm",
                                onclick: on_parse_click,
                                "Parse"
                            }
                            button {
                                class: "btn btn--danger btn--sm",
                                disabled: !parse_ready,
                                onclick: confirm_delete,
                                "Delete {total} row(s)"
                            }
                        }
                    }
                }
            }
        }
        None => rsx! {},
    };

    let data_cls = if drawer_open { "data" } else { "data data--no-drawer" };

    rsx! {
        section { class: "page page--flush", onmounted: on_mount,
            div { class: data_cls,
                TablesPane {
                    tables: tables.clone(),
                    selected: selected_table.clone(),
                    writer_tables,
                    row_counts: row_counts_by_table,
                    on_select: move |name| select_table(name),
                }
                {partitions_content}
                {center_content}
                {drawer_content}
            }
            {dialog_render}
        }
    }
}

fn parse_paste_delete_input(
    raw: &str,
) -> Result<(BTreeMap<String, Vec<String>>, usize, usize), String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Input is empty.".to_string());
    }

    let parsed: Value = serde_json::from_str(trimmed)
        .map_err(|err| format!("Invalid JSON: {}", err))?;

    let arr = parsed
        .as_array()
        .ok_or_else(|| "Top-level JSON must be an array.".to_string())?;

    if arr.is_empty() {
        return Err("Array is empty.".to_string());
    }

    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut total: usize = 0;

    for (idx, item) in arr.iter().enumerate() {
        let obj = item.as_object().ok_or_else(|| {
            format!(
                "Item #{} is not an object (expected {{ PartitionKey, RowKey }}).",
                idx
            )
        })?;

        let pk = obj
            .get(PARTITION_KEY)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                format!(
                    "Item #{} is missing a string \"PartitionKey\" field.",
                    idx
                )
            })?;
        let rk = obj
            .get(ROW_KEY)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                format!("Item #{} is missing a string \"RowKey\" field.", idx)
            })?;

        if pk.is_empty() {
            return Err(format!("Item #{}: PartitionKey is empty.", idx));
        }
        if rk.is_empty() {
            return Err(format!("Item #{}: RowKey is empty.", idx));
        }

        grouped
            .entry(pk.to_string())
            .or_insert_with(Vec::new)
            .push(rk.to_string());
        total += 1;
    }

    let partitions_touched = grouped.len();
    Ok((grouped, total, partitions_touched))
}

fn build_rows_state(rows: Vec<Value>) -> (Vec<String>, Vec<Value>) {
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

    (headers, rows)
}

fn build_writer_tables(status: &Option<StatusApiModel>) -> HashSet<String> {
    let mut set = HashSet::new();
    if let Some(s) = status {
        if let Some(init) = &s.initialized {
            for w in &init.writers {
                for t in &w.tables {
                    set.insert(t.clone());
                }
            }
        }
    }
    set
}

fn derive_table_connectivity(
    status: &Option<StatusApiModel>,
    table: &str,
) -> (Vec<String>, usize) {
    let mut writers = Vec::new();
    let mut readers = 0usize;
    if let Some(s) = status {
        if let Some(init) = &s.initialized {
            for w in &init.writers {
                if w.tables.iter().any(|t| t == table) {
                    writers.push(w.name.clone());
                }
            }
            for r in &init.readers {
                if r.is_node {
                    continue;
                }
                if r.tables.iter().any(|t| t == table) {
                    readers += 1;
                }
            }
        }
    }
    (writers, readers)
}

fn derive_table_stats(status: &Option<StatusApiModel>, table: &str) -> Option<TableApiModel> {
    status
        .as_ref()
        .and_then(|s| s.initialized.as_ref())
        .and_then(|init| init.tables.iter().find(|t| t.name == table).cloned())
}

fn derive_table_row_counts(status: &Option<StatusApiModel>) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    if let Some(s) = status {
        if let Some(init) = &s.initialized {
            for t in &init.tables {
                map.insert(t.name.clone(), t.records_amount);
            }
        }
    }
    map
}

fn render_empty_state(
    tables: Vec<TableListItemApiModel>,
    on_pick: impl FnMut(String) + Clone + 'static,
) -> Element {
    let chips = tables.into_iter().take(8).map(|t| {
        let name = t.name.clone();
        let mut on_pick = on_pick.clone();
        rsx! {
            button {
                class: "btn btn--sm",
                onclick: move |_| on_pick(name.clone()),
                Badge { text: t.name.clone(), tone: BadgeTone::Neutral }
            }
        }
    });

    rsx! {
        div { class: "rows-col",
            div { class: "empty-state",
                div { class: "empty-state__icon",
                    Icon { kind: IconKind::Layers }
                }
                div { class: "empty-state__title", "Select a table to begin" }
                div { class: "empty-state__sub", "Choose a table from the left, or pick one of the recently active tables below." }
                div { class: "empty-state__chips", {chips} }
            }
        }
    }
}
