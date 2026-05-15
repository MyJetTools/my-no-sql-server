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
    #[route("/data")]
    Data {},
    #[route("/snapshots")]
    Snapshots {},
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
    let section = match route {
        AppRoute::Home {} => SidebarSection::Overview,
        AppRoute::Data {} => SidebarSection::Tables,
        AppRoute::Snapshots {} => SidebarSection::Snapshots,
        AppRoute::Settings {} => SidebarSection::Settings,
        _ => SidebarSection::Overview,
    };

    let crumbs = match section {
        SidebarSection::Overview => vec![
            Crumb { label: "MyNoSql".to_string(), active: false },
            Crumb { label: "Overview".to_string(), active: true },
        ],
        SidebarSection::Tables => vec![
            Crumb { label: "MyNoSql".to_string(), active: false },
            Crumb { label: "Tables".to_string(), active: true },
        ],
        SidebarSection::Snapshots => vec![
            Crumb { label: "MyNoSql".to_string(), active: false },
            Crumb { label: "Snapshots".to_string(), active: true },
        ],
        SidebarSection::Settings => vec![
            Crumb { label: "MyNoSql".to_string(), active: false },
            Crumb { label: "Settings".to_string(), active: true },
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
