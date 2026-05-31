use dioxus::prelude::*;
use serde_json::Value;

use crate::AppRoute;
use crate::api;
use crate::components::data::{PARTITION_KEY, ROW_KEY, RowsTable, TIME_STAMP};
use crate::models::{SnapshotFileApiModel, SnapshotTableApiModel};

#[derive(Default)]
struct SnapshotsState {
    // Snapshot files list (loaded once, refreshed on demand).
    files: Vec<SnapshotFileApiModel>,
    files_started: bool,
    files_ready: bool,

    // Tables for the selected file.
    loaded_for_file: Option<String>,
    tables: Vec<SnapshotTableApiModel>,
    tables_ready: bool,

    // Partitions for the selected (file, table).
    loaded_for_table: Option<(String, String)>,
    partitions: Vec<String>,
    partitions_ready: bool,

    // Rows for the selected (file, table, partition).
    loaded_rows_for: Option<(String, String, String)>,
    row_headers: Vec<String>,
    rows: Vec<Value>,
    rows_ready: bool,

    error: Option<String>,
}

impl SnapshotsState {
    fn begin_files_load(&mut self) {
        self.files_started = true;
        self.files_ready = false;
        self.error = None;
    }

    fn set_files(&mut self, files: Vec<SnapshotFileApiModel>) {
        self.files = files;
        self.files_ready = true;
    }

    fn begin_tables_load(&mut self, file: &str) {
        self.loaded_for_file = Some(file.to_string());
        self.tables = Vec::new();
        self.tables_ready = false;
        self.error = None;
    }

    fn set_tables(&mut self, file: &str, tables: Vec<SnapshotTableApiModel>) {
        if self.loaded_for_file.as_deref() == Some(file) {
            self.tables = tables;
            self.tables_ready = true;
        }
    }

    fn begin_partitions_load(&mut self, key: &(String, String)) {
        self.loaded_for_table = Some(key.clone());
        self.partitions = Vec::new();
        self.partitions_ready = false;
        self.error = None;
    }

    fn set_partitions(&mut self, key: &(String, String), partitions: Vec<String>) {
        if self.loaded_for_table.as_ref() == Some(key) {
            self.partitions = partitions;
            self.partitions_ready = true;
        }
    }

    fn begin_rows_load(&mut self, key: &(String, String, String)) {
        self.loaded_rows_for = Some(key.clone());
        self.row_headers = Vec::new();
        self.rows = Vec::new();
        self.rows_ready = false;
        self.error = None;
    }

    fn set_rows(&mut self, key: &(String, String, String), headers: Vec<String>, rows: Vec<Value>) {
        if self.loaded_rows_for.as_ref() == Some(key) {
            self.row_headers = headers;
            self.rows = rows;
            self.rows_ready = true;
        }
    }
}

/// Extract `(file, table, partition)` from the current snapshot route.
fn parse_snapshot_route(route: &AppRoute) -> (Option<String>, Option<String>, Option<String>) {
    match route {
        AppRoute::SnapshotFile { file } => (Some(file.clone()), None, None),
        AppRoute::SnapshotTable { file, table } => {
            (Some(file.clone()), Some(table.clone()), None)
        }
        AppRoute::SnapshotPartition {
            file,
            table,
            partition,
        } => (
            Some(file.clone()),
            Some(table.clone()),
            Some(partition.clone()),
        ),
        _ => (None, None, None),
    }
}

// Route placeholders — the URL patterns for the snapshots section. `SnapshotsLayout`
// renders the whole page and reads the params via `use_route`, so these render
// nothing themselves.
#[component]
pub fn Snapshots() -> Element {
    rsx! {}
}

#[component]
pub fn SnapshotFile(file: String) -> Element {
    let _ = file;
    rsx! {}
}

#[component]
pub fn SnapshotTable(file: String, table: String) -> Element {
    let _ = (file, table);
    rsx! {}
}

#[component]
pub fn SnapshotPartition(file: String, table: String, partition: String) -> Element {
    let _ = (file, table, partition);
    rsx! {}
}

#[component]
pub fn SnapshotsLayout() -> Element {
    let mut cs = use_signal(SnapshotsState::default);
    let nav = navigator();

    let route = use_route::<AppRoute>();
    let (url_file, url_table, url_partition) = parse_snapshot_route(&route);

    // ---- load files list ----
    if !cs.read().files_started {
        spawn(async move {
            if cs.peek().files_started {
                return;
            }
            cs.write().begin_files_load();
            match api::get_snapshots_list().await {
                Ok(mut files) => {
                    files.sort_by(|a, b| b.name.cmp(&a.name));
                    cs.write().set_files(files);
                }
                Err(err) => {
                    cs.write().error = Some(format!("Failed to load snapshots: {}", err));
                }
            }
        });
    }

    // ---- load tables whenever the URL file changes ----
    if let Some(file) = url_file.clone() {
        if cs.read().loaded_for_file.as_deref() != Some(file.as_str()) {
            spawn(async move {
                if cs.peek().loaded_for_file.as_deref() == Some(file.as_str()) {
                    return;
                }
                cs.write().begin_tables_load(&file);
                match api::get_snapshot_tables(&file).await {
                    Ok(tables) => cs.write().set_tables(&file, tables),
                    Err(err) => {
                        cs.write().error = Some(format!("Failed to load tables: {}", err));
                    }
                }
            });
        }
    }

    // ---- load partitions whenever the URL (file, table) changes ----
    if let (Some(file), Some(table)) = (url_file.clone(), url_table.clone()) {
        let key = (file, table);
        if cs.read().loaded_for_table.as_ref() != Some(&key) {
            spawn(async move {
                if cs.peek().loaded_for_table.as_ref() == Some(&key) {
                    return;
                }
                cs.write().begin_partitions_load(&key);
                match api::get_snapshot_partitions(&key.0, &key.1).await {
                    Ok(partitions) => cs.write().set_partitions(&key, partitions),
                    Err(err) => {
                        cs.write().error = Some(format!("Failed to load partitions: {}", err));
                    }
                }
            });
        }
    }

    // ---- load rows whenever the URL (file, table, partition) changes ----
    if let (Some(file), Some(table), Some(pk)) =
        (url_file.clone(), url_table.clone(), url_partition.clone())
    {
        let key = (file, table, pk);
        if cs.read().loaded_rows_for.as_ref() != Some(&key) {
            spawn(async move {
                if cs.peek().loaded_rows_for.as_ref() == Some(&key) {
                    return;
                }
                cs.write().begin_rows_load(&key);
                match api::get_snapshot_rows(&key.0, &key.1, &key.2).await {
                    Ok(rows) => {
                        let (headers, rows) = build_rows_state(rows);
                        cs.write().set_rows(&key, headers, rows);
                    }
                    Err(err) => {
                        cs.write().error = Some(format!("Failed to load rows: {}", err));
                    }
                }
            });
        }
    }

    // ---- snapshot of state for rendering ----
    let cs_ra = cs.read();
    let error = cs_ra.error.clone();
    let files = cs_ra.files.clone();
    let files_ready = cs_ra.files_ready;

    let tables_scope = url_file.is_some() && cs_ra.loaded_for_file.as_deref() == url_file.as_deref();
    let tables = if tables_scope { cs_ra.tables.clone() } else { Vec::new() };
    let tables_ready = tables_scope && cs_ra.tables_ready;

    let partitions_key = match (&url_file, &url_table) {
        (Some(f), Some(t)) => Some((f.clone(), t.clone())),
        _ => None,
    };
    let partitions_scope =
        partitions_key.is_some() && cs_ra.loaded_for_table.as_ref() == partitions_key.as_ref();
    let partitions = if partitions_scope {
        cs_ra.partitions.clone()
    } else {
        Vec::new()
    };
    let partitions_ready = partitions_scope && cs_ra.partitions_ready;

    let rows_key = match (&url_file, &url_table, &url_partition) {
        (Some(f), Some(t), Some(p)) => Some((f.clone(), t.clone(), p.clone())),
        _ => None,
    };
    let rows_scope = rows_key.is_some() && cs_ra.loaded_rows_for.as_ref() == rows_key.as_ref();
    let row_headers = if rows_scope {
        cs_ra.row_headers.clone()
    } else {
        Vec::new()
    };
    let rows = if rows_scope { cs_ra.rows.clone() } else { Vec::new() };
    let rows_ready = rows_scope && cs_ra.rows_ready;
    drop(cs_ra);

    // Refresh re-fetches the deepest level matching the current URL by clearing
    // its load marker — the inline loaders above pick it up on the next render.
    let refresh_file = url_file.clone();
    let refresh_table = url_table.clone();
    let refresh_partition = url_partition.clone();
    let on_refresh = move |_| {
        let mut w = cs.write();
        match (
            refresh_file.is_some(),
            refresh_table.is_some(),
            refresh_partition.is_some(),
        ) {
            (false, _, _) => w.files_started = false,
            (true, false, _) => w.loaded_for_file = None,
            (true, true, false) => w.loaded_for_table = None,
            (true, true, true) => w.loaded_rows_for = None,
        }
    };
    let loading = match (&url_file, &url_table, &url_partition) {
        (None, _, _) => !files_ready,
        (Some(_), None, _) => !tables_ready,
        (Some(_), Some(_), None) => !partitions_ready,
        (Some(_), Some(_), Some(_)) => !rows_ready,
    };

    // ---- navigation handlers ----
    let open_file = move |file: String| {
        nav.push(AppRoute::SnapshotFile { file });
    };
    let open_table = {
        let file = url_file.clone();
        move |table: String| {
            if let Some(file) = file.clone() {
                nav.push(AppRoute::SnapshotTable { file, table });
            }
        }
    };
    let open_partition = {
        let file = url_file.clone();
        let table = url_table.clone();
        move |partition: String| {
            if let (Some(file), Some(table)) = (file.clone(), table.clone()) {
                nav.push(AppRoute::SnapshotPartition {
                    file,
                    table,
                    partition,
                });
            }
        }
    };

    let go_to_files = move |_| {
        nav.push(AppRoute::Snapshots {});
    };
    let go_to_tables = {
        let file = url_file.clone();
        move |_| {
            if let Some(file) = file.clone() {
                nav.push(AppRoute::SnapshotFile { file });
            }
        }
    };
    let go_to_partitions = {
        let file = url_file.clone();
        let table = url_table.clone();
        move |_| {
            if let (Some(file), Some(table)) = (file.clone(), table.clone()) {
                nav.push(AppRoute::SnapshotTable { file, table });
            }
        }
    };

    let error_view = if let Some(err) = error {
        rsx! {
            div { style: "color: var(--danger); font-size: 12.5px; padding: 8px 0;", "{err}" }
        }
    } else {
        rsx! {}
    };

    let body = match (
        url_file.clone(),
        url_table.clone(),
        url_partition.clone(),
    ) {
        (None, _, _) => render_files(files, !files_ready, open_file),
        (Some(_), None, _) => render_tables(tables, !tables_ready, open_table),
        (Some(_), Some(_), None) => render_partitions(partitions, !partitions_ready, open_partition),
        (Some(_), Some(_), Some(_)) => render_rows(row_headers, rows, !rows_ready),
    };

    let crumbs = render_crumbs(
        url_file.clone(),
        url_table.clone(),
        url_partition.clone(),
        go_to_files,
        go_to_tables,
        go_to_partitions,
    );

    rsx! {
        section { class: "page page--padded",
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
        Outlet::<AppRoute> {}
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
    files: Vec<SnapshotFileApiModel>,
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
    let rows = files.into_iter().map(move |file| {
        let n = file.name.clone();
        let size = crate::utils::format_bytes(file.size as f64);
        let mut on_pick = on_pick.clone();
        rsx! {
            tr {
                key: "{file.name}",
                style: "cursor: pointer;",
                onclick: move |_| on_pick(n.clone()),
                td { style: "font-family: var(--font-mono);", "{file.name}" }
                td { class: "num", "{size}" }
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
                    thead {
                        tr {
                            th { "File name" }
                            th { class: "num", "Size" }
                        }
                    }
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
    let rows = tables.into_iter().map(move |t| {
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
    let rows = partitions.into_iter().map(move |pk| {
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
