use dioxus::prelude::*;

use crate::AppRoute;
use crate::components::atoms::{Icon, IconKind};

#[component]
pub fn Sidebar(active: SidebarSection, tables_count: usize, clients_count: usize, online: bool) -> Element {
    let dot_class = if online { "sidebar__live-dot" } else { "sidebar__live-dot offline" };
    let live_text = if online {
        format!("Live · {} clients", clients_count)
    } else {
        "Offline".to_string()
    };

    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar__brand",
                div { class: "sidebar__logo", "N" }
                div {
                    div { class: "sidebar__brand-name", "MyNoSql" }
                    div { class: "sidebar__brand-sub", "v0.7.3 · prod" }
                }
            }
            nav { class: "sidebar__nav",
                Link {
                    to: AppRoute::Home {},
                    class: nav_class(active == SidebarSection::Overview),
                    Icon { kind: IconKind::Activity, class: "sidebar__nav-icon".to_string() }
                    span { class: "sidebar__nav-label", "Overview" }
                }
                Link {
                    to: AppRoute::Data {},
                    class: nav_class(active == SidebarSection::Tables),
                    Icon { kind: IconKind::Database, class: "sidebar__nav-icon".to_string() }
                    span { class: "sidebar__nav-label", "Tables" }
                    span { class: "sidebar__nav-count", "{tables_count}" }
                }
                div { class: "sidebar__nav-item",
                    Icon { kind: IconKind::Plug, class: "sidebar__nav-icon".to_string() }
                    span { class: "sidebar__nav-label", "Connections" }
                    span { class: "sidebar__nav-count", "{clients_count}" }
                }
                div { class: "sidebar__nav-item",
                    Icon { kind: IconKind::HardDrive, class: "sidebar__nav-icon".to_string() }
                    span { class: "sidebar__nav-label", "Persistence" }
                }
                div { class: "sidebar__nav-item",
                    Icon { kind: IconKind::Settings, class: "sidebar__nav-icon".to_string() }
                    span { class: "sidebar__nav-label", "Settings" }
                }
            }
            div { class: "sidebar__foot",
                span { class: dot_class }
                span { "{live_text}" }
            }
        }
    }
}

fn nav_class(active: bool) -> &'static str {
    if active {
        "sidebar__nav-item active"
    } else {
        "sidebar__nav-item"
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SidebarSection {
    Overview,
    Tables,
}
