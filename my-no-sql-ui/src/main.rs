use dioxus::prelude::*;

mod api;
mod components;
mod models;
mod pages;
mod settings;
mod storage;
mod utils;

use components::shell::{Crumb, Sidebar, SidebarSection, Topbar};
use models::StatusApiModel;
use pages::*;
use settings::HealthThresholds;

#[derive(Routable, PartialEq, Clone)]
pub enum AppRoute {
    #[layout(Shell)]
    #[route("/")]
    Home {},
    #[layout(DataLayout)]
    #[route("/data")]
    Data {},
    #[route("/data/:table")]
    DataTable { table: String },
    #[route("/data/:table/:partition")]
    DataPartition { table: String, partition: String },
    #[route("/data/:table/:partition/:row")]
    DataRow {
        table: String,
        partition: String,
        row: String,
    },
    #[end_layout]
    #[route("/connections")]
    Connections {},
    #[layout(SnapshotsLayout)]
    #[route("/snapshots")]
    Snapshots {},
    #[route("/snapshots/:file")]
    SnapshotFile { file: String },
    #[route("/snapshots/:file/:table")]
    SnapshotTable { file: String, table: String },
    #[route("/snapshots/:file/:table/:partition")]
    SnapshotPartition {
        file: String,
        table: String,
        partition: String,
    },
    #[end_layout]
    #[route("/settings")]
    Settings {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[derive(Clone, Default)]
pub struct AppContext {
    pub status: Option<StatusApiModel>,
    pub refresh_token: u64,
}

fn main() {
    dioxus::LaunchBuilder::new().launch(|| {
        let theme = storage::load_theme().unwrap_or_else(|| "light".to_string());
        storage::apply_theme(&theme);

        rsx! {
            document::Link { rel: "icon", href: asset!("/public/favicon.ico") }
            Router::<AppRoute> {}
        }
    });
}

#[component]
fn Shell() -> Element {
    let ctx_signal = use_context_provider(|| Signal::new(AppContext::default()));
    let mut ctx = ctx_signal;

    // Health thresholds (Green/Yellow/Red) — loaded from the server once,
    // edited via the Settings page, persisted server-side.
    let thresholds_signal: Signal<HealthThresholds> =
        use_context_provider(|| Signal::new(HealthThresholds::default()));
    let mut thresholds = thresholds_signal;
    let mut thresholds_loaded = use_signal(|| false);
    let loaded_val = *thresholds_loaded.read();
    use_effect(move || {
        if loaded_val {
            return;
        }
        *thresholds_loaded.write() = true;
        spawn(async move {
            if let Ok(t) = api::get_health_thresholds().await {
                thresholds.set(t);
            }
        });
    });

    let route = use_route::<AppRoute>();
    let section = match &route {
        AppRoute::Home {} => SidebarSection::Overview,
        AppRoute::Data {}
        | AppRoute::DataTable { .. }
        | AppRoute::DataPartition { .. }
        | AppRoute::DataRow { .. } => SidebarSection::Tables,
        AppRoute::Connections {} => SidebarSection::Connections,
        AppRoute::Snapshots {}
        | AppRoute::SnapshotFile { .. }
        | AppRoute::SnapshotTable { .. }
        | AppRoute::SnapshotPartition { .. } => SidebarSection::Snapshots,
        AppRoute::Settings {} => SidebarSection::Settings,
        _ => SidebarSection::Overview,
    };

    let crumbs = match &route {
        AppRoute::Connections {} => vec![
            Crumb { label: "MyNoSql".to_string(), active: false },
            Crumb { label: "Connections".to_string(), active: true },
        ],
        AppRoute::Snapshots {}
        | AppRoute::SnapshotFile { .. }
        | AppRoute::SnapshotTable { .. }
        | AppRoute::SnapshotPartition { .. } => vec![
            Crumb { label: "MyNoSql".to_string(), active: false },
            Crumb { label: "Snapshots".to_string(), active: true },
        ],
        AppRoute::Settings {} => vec![
            Crumb { label: "MyNoSql".to_string(), active: false },
            Crumb { label: "Settings".to_string(), active: true },
        ],
        AppRoute::Data {}
        | AppRoute::DataTable { .. }
        | AppRoute::DataPartition { .. }
        | AppRoute::DataRow { .. } => build_data_crumbs(&route),
        AppRoute::Home {} | AppRoute::NotFound { .. } => vec![
            Crumb { label: "MyNoSql".to_string(), active: false },
            Crumb { label: "Overview".to_string(), active: true },
        ],
    };

    let ctx_ra = ctx.read();
    let status = ctx_ra.status.clone();
    drop(ctx_ra);

    let online = status.is_some();
    let (tables_count, clients_count) = if let Some(s) = status.as_ref() {
        let tables = s.initialized.as_ref().map(|i| i.tables.len()).unwrap_or(0);
        let clients = s.initialized.as_ref().map(|i| i.readers.len() + i.writers.len()).unwrap_or(0);
        (tables, clients)
    } else {
        (0, 0)
    };

    let on_refresh = move |_| {
        let next = ctx.read().refresh_token.wrapping_add(1);
        ctx.write().refresh_token = next;
    };

    rsx! {
        div { class: "shell",
            Sidebar { active: section, tables_count, clients_count, online }
            div { class: "main",
                Topbar { crumbs, on_refresh: on_refresh }
                Outlet::<AppRoute> {}
            }
        }
    }
}

/// Breadcrumbs for the data routes — reflects the `/data/<table>/<partition>/<row>`
/// path, with the deepest selected segment marked active.
fn build_data_crumbs(route: &AppRoute) -> Vec<Crumb> {
    let (table, partition, row) = match route {
        AppRoute::DataTable { table } => (Some(table.clone()), None, None),
        AppRoute::DataPartition { table, partition } => {
            (Some(table.clone()), Some(partition.clone()), None)
        }
        AppRoute::DataRow {
            table,
            partition,
            row,
        } => (
            Some(table.clone()),
            Some(partition.clone()),
            Some(row.clone()),
        ),
        _ => (None, None, None),
    };

    let mut crumbs = vec![Crumb { label: "MyNoSql".to_string(), active: false }];
    crumbs.push(Crumb { label: "Tables".to_string(), active: table.is_none() });
    if let Some(t) = table {
        crumbs.push(Crumb { label: t, active: partition.is_none() });
    }
    if let Some(p) = partition {
        crumbs.push(Crumb { label: p, active: row.is_none() });
    }
    if let Some(r) = row {
        crumbs.push(Crumb { label: r, active: true });
    }
    crumbs
}
