use dioxus::prelude::*;
use serde_json::Value;

use crate::api;
use crate::components::data::{PARTITION_KEY, ROW_KEY, RowsTable, TIME_STAMP};
use crate::models::SnapshotTableApiModel;

#[derive(Default)]
struct SnapshotsState {
    started: bool,
    loading: bool,
    error: Option<String>,

    files: Vec<String>,

    selected_file: Option<String>,
    tables: Vec<SnapshotTableApiModel>,

    selected_table: Option<String>,
    partitions: Vec<String>,

    selected_partition: Option<String>,
    row_headers: Vec<String>,
    rows: Vec<Value>,
}

#[component]
pub fn Snapshots() -> Element {
    let mut cs = use_signal(SnapshotsState::default);

    let mut load_files = move || {
        {
            let mut w = cs.write();
            w.loading = true;
            w.error = None;
        }
        spawn(async move {
            match api::get_snapshots_list().await {
                Ok(mut files) => {
                    files.sort_by(|a, b| b.cmp(a));
                    let mut w = cs.write();
                    w.files = files;
                    w.loading = false;
                }
                Err(err) => {
                    let mut w = cs.write();
                    w.loading = false;
                    w.error = Some(format!("Failed to load snapshots: {}", err));
                }
            }
        });
    };

    let mut open_file = move |file: String| {
        {
            let mut w = cs.write();
            w.selected_file = Some(file.clone());
            w.tables = Vec::new();
            w.selected_table = None;
            w.partitions = Vec::new();
            w.selected_partition = None;
            w.rows = Vec::new();
            w.row_headers = Vec::new();
            w.loading = true;
            w.error = None;
        }
        spawn(async move {
            match api::get_snapshot_tables(&file).await {
                Ok(tables) => {
                    let mut w = cs.write();
                    w.tables = tables;
                    w.loading = false;
                }
                Err(err) => {
                    let mut w = cs.write();
                    w.loading = false;
                    w.error = Some(format!("Failed to load tables: {}", err));
                }
            }
        });
    };

    let mut open_table = move |table_name: String| {
        let file = match cs.read().selected_file.clone() {
            Some(f) => f,
            None => return,
        };
        {
            let mut w = cs.write();
            w.selected_table = Some(table_name.clone());
            w.partitions = Vec::new();
            w.selected_partition = None;
            w.rows = Vec::new();
            w.row_headers = Vec::new();
            w.loading = true;
            w.error = None;
        }
        spawn(async move {
            match api::get_snapshot_partitions(&file, &table_name).await {
                Ok(partitions) => {
                    let mut w = cs.write();
                    w.partitions = partitions;
                    w.loading = false;
                }
                Err(err) => {
                    let mut w = cs.write();
                    w.loading = false;
                    w.error = Some(format!("Failed to load partitions: {}", err));
                }
            }
        });
    };

    let mut open_partition = move |partition_key: String| {
        let (file, table) = {
            let r = cs.read();
            match (r.selected_file.clone(), r.selected_table.clone()) {
                (Some(f), Some(t)) => (f, t),
                _ => return,
            }
        };
        {
            let mut w = cs.write();
            w.selected_partition = Some(partition_key.clone());
            w.rows = Vec::new();
            w.row_headers = Vec::new();
            w.loading = true;
            w.error = None;
        }
        spawn(async move {
            match api::get_snapshot_rows(&file, &table, &partition_key).await {
                Ok(rows) => {
                    let (headers, rows) = build_rows_state(rows);
                    let mut w = cs.write();
                    w.row_headers = headers;
                    w.rows = rows;
                    w.loading = false;
                }
                Err(err) => {
                    let mut w = cs.write();
                    w.loading = false;
                    w.error = Some(format!("Failed to load rows: {}", err));
                }
            }
        });
    };

    let on_mount = move |_| {
        if cs.read().started {
            return;
        }
        cs.write().started = true;
        load_files();
    };

    let mut go_to_files = move || {
        let mut w = cs.write();
        w.selected_file = None;
        w.selected_table = None;
        w.selected_partition = None;
        w.tables = Vec::new();
        w.partitions = Vec::new();
        w.rows = Vec::new();
        w.row_headers = Vec::new();
        w.error = None;
    };

    let mut go_to_tables = move || {
        let mut w = cs.write();
        w.selected_table = None;
        w.selected_partition = None;
        w.partitions = Vec::new();
        w.rows = Vec::new();
        w.row_headers = Vec::new();
        w.error = None;
    };

    let mut go_to_partitions = move || {
        let mut w = cs.write();
        w.selected_partition = None;
        w.rows = Vec::new();
        w.row_headers = Vec::new();
        w.error = None;
    };

    // Snapshot of state for rendering
    let cs_ra = cs.read();
    let loading = cs_ra.loading;
    let error = cs_ra.error.clone();
    let files = cs_ra.files.clone();
    let selected_file = cs_ra.selected_file.clone();
    let tables = cs_ra.tables.clone();
    let selected_table = cs_ra.selected_table.clone();
    let partitions = cs_ra.partitions.clone();
    let selected_partition = cs_ra.selected_partition.clone();
    let row_headers = cs_ra.row_headers.clone();
    let rows = cs_ra.rows.clone();
    drop(cs_ra);

    let error_view = if let Some(err) = error.clone() {
        rsx! {
            div { style: "color: var(--danger); font-size: 12.5px; padding: 8px 0;", "{err}" }
        }
    } else {
        rsx! {}
    };

    let body = match (
        selected_file.clone(),
        selected_table.clone(),
        selected_partition.clone(),
    ) {
        (None, _, _) => render_files(files, loading, move |name| open_file(name)),
        (Some(_file), None, _) => render_tables(tables, loading, move |name| open_table(name)),
        (Some(_file), Some(_table), None) => {
            render_partitions(partitions, loading, move |pk| open_partition(pk))
        }
        (Some(_file), Some(_table), Some(_pk)) => render_rows(row_headers, rows, loading),
    };

    let crumbs = render_crumbs(
        selected_file.clone(),
        selected_table.clone(),
        selected_partition.clone(),
        move |_| go_to_files(),
        move |_| go_to_tables(),
        move |_| go_to_partitions(),
    );

    let on_refresh = move |_| {
        let (file, table, pk) = {
            let r = cs.read();
            (
                r.selected_file.clone(),
                r.selected_table.clone(),
                r.selected_partition.clone(),
            )
        };
        match (file, table, pk) {
            (None, _, _) => load_files(),
            (Some(f), None, _) => open_file(f),
            (Some(_), Some(t), None) => open_table(t),
            (Some(_), Some(_), Some(pk)) => open_partition(pk),
        }
    };

    rsx! {
        section { class: "page page--padded", onmounted: on_mount,
            div { style: "display: flex; flex-direction: column; gap: 14px; max-width: 960px;",
                div { style: "display: flex; align-items: center; justify-content: space-between; gap: 12px;",
                    {crumbs}
                    button {
                        class: "btn btn--ghost btn--sm",
                        disabled: loading,
                        onclick: on_refresh,
                        if loading { "Refreshing…" } else { "Refresh" }
                    }
                }
                {error_view}
                {body}
            }
        }
    }
}

fn render_crumbs(
    file: Option<String>,
    table: Option<String>,
    partition: Option<String>,
    on_root: impl FnMut(()) + Clone + 'static,
    on_file: impl FnMut(()) + Clone + 'static,
    on_table: impl FnMut(()) + Clone + 'static,
) -> Element {
    let mut on_root = on_root;
    let mut on_file = on_file;
    let mut on_table = on_table;

    let file_segment = file.clone();
    let table_segment = table.clone();
    let partition_segment = partition.clone();

    let file_active = file.is_some();
    let table_active = table.is_some();
    let partition_active = partition.is_some();

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: 6px; font-size: 13px; color: var(--text-muted);",
            button {
                class: "btn btn--ghost btn--sm",
                disabled: !file_active,
                onclick: move |_| on_root(()),
                "Snapshots"
            }
            if let Some(name) = file_segment {
                span { "›" }
                button {
                    class: "btn btn--ghost btn--sm",
                    style: "font-family: var(--font-mono);",
                    disabled: !table_active,
                    onclick: move |_| on_file(()),
                    "{name}"
                }
            }
            if let Some(name) = table_segment {
                span { "›" }
                button {
                    class: "btn btn--ghost btn--sm",
                    style: "font-family: var(--font-mono);",
                    disabled: !partition_active,
                    onclick: move |_| on_table(()),
                    "{name}"
                }
            }
            if let Some(name) = partition_segment {
                span { "›" }
                span { style: "font-family: var(--font-mono); padding: 0 6px;", "{name}" }
            }
        }
    }
}

fn render_files(
    files: Vec<String>,
    loading: bool,
    on_pick: impl FnMut(String) + Clone + 'static,
) -> Element {
    if loading && files.is_empty() {
        return rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "Loading snapshots…" }
            }
        };
    }
    if files.is_empty() {
        return rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "No snapshots yet" }
                div { class: "empty-state__sub",
                    "Snapshots are written periodically into the backup folder configured on the server."
                }
            }
        };
    }
    let count = files.len();
    let rows =files.into_iter().map(move |name| {
        let n = name.clone();
        let mut on_pick = on_pick.clone();
        rsx! {
            tr {
                key: "{name}",
                style: "cursor: pointer;",
                onclick: move |_| on_pick(n.clone()),
                td { style: "font-family: var(--font-mono);", "{name}" }
            }
        }
    });
    rsx! {
        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Snapshot files" }
                span { class: "card__subtitle", "{count} file(s)" }
            }
            div { class: "card__body",
                table { class: "rt",
                    thead { tr { th { "File name" } } }
                    tbody { {rows} }
                }
            }
        }
    }
}

fn render_tables(
    tables: Vec<SnapshotTableApiModel>,
    loading: bool,
    on_pick: impl FnMut(String) + Clone + 'static,
) -> Element {
    if loading && tables.is_empty() {
        return rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "Loading tables…" }
            }
        };
    }
    if tables.is_empty() {
        return rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "No tables in this snapshot" }
            }
        };
    }
    let count = tables.len();
    let rows =tables.into_iter().map(move |t| {
        let n = t.name.clone();
        let mut on_pick = on_pick.clone();
        rsx! {
            tr {
                key: "{t.name}",
                style: "cursor: pointer;",
                onclick: move |_| on_pick(n.clone()),
                td { style: "font-family: var(--font-mono);", "{t.name}" }
                td { class: "num", "{t.partitions_count}" }
            }
        }
    });
    rsx! {
        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Tables" }
                span { class: "card__subtitle", "{count} table(s)" }
            }
            div { class: "card__body",
                table { class: "rt",
                    thead {
                        tr {
                            th { "Table name" }
                            th { class: "num", "Partitions" }
                        }
                    }
                    tbody { {rows} }
                }
            }
        }
    }
}

fn render_partitions(
    partitions: Vec<String>,
    loading: bool,
    on_pick: impl FnMut(String) + Clone + 'static,
) -> Element {
    if loading && partitions.is_empty() {
        return rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "Loading partitions…" }
            }
        };
    }
    if partitions.is_empty() {
        return rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "No partitions in this table" }
            }
        };
    }
    let count = partitions.len();
    let rows =partitions.into_iter().map(move |pk| {
        let p = pk.clone();
        let mut on_pick = on_pick.clone();
        rsx! {
            tr {
                key: "{pk}",
                style: "cursor: pointer;",
                onclick: move |_| on_pick(p.clone()),
                td { style: "font-family: var(--font-mono);", "{pk}" }
            }
        }
    });
    rsx! {
        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Partitions" }
                span { class: "card__subtitle", "{count} partition(s)" }
            }
            div { class: "card__body",
                table { class: "rt",
                    thead { tr { th { "Partition key" } } }
                    tbody { {rows} }
                }
            }
        }
    }
}

fn render_rows(headers: Vec<String>, rows: Vec<Value>, loading: bool) -> Element {
    if loading && rows.is_empty() {
        return rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "Loading rows…" }
            }
        };
    }
    let count = rows.len();
    rsx! {
        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Rows" }
                span { class: "card__subtitle", "{count} row(s)" }
            }
            div { class: "card__body", style: "padding: 0;",
                RowsTable {
                    headers,
                    rows,
                    selected_row_key: None::<String>,
                    on_row_click: |_| {},
                }
            }
        }
    }
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
