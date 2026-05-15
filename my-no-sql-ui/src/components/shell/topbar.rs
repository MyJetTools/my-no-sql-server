use dioxus::prelude::*;

use crate::components::atoms::{Icon, IconKind};
use crate::storage;

#[derive(Clone, PartialEq)]
pub struct Crumb {
    pub label: String,
    pub active: bool,
}

#[component]
pub fn Topbar(crumbs: Vec<Crumb>, on_refresh: EventHandler<()>) -> Element {
    let mut theme = use_signal(|| storage::load_theme().unwrap_or_else(|| "light".to_string()));
    let theme_val = theme.read().clone();
    let is_dark = theme_val == "dark";

    let toggle_theme = move |_| {
        let next = if theme.read().as_str() == "dark" { "light" } else { "dark" };
        storage::save_theme(next);
        storage::apply_theme(next);
        theme.set(next.to_string());
    };

    let crumbs_iter = crumbs.into_iter().enumerate().map(|(i, c)| {
        let cls = if c.active { "topbar__crumb active" } else { "topbar__crumb" };
        let sep = if i > 0 {
            rsx! {
                span { class: "topbar__crumb-sep", "/" }
            }
        } else {
            rsx! {}
        };
        rsx! {
            {sep}
            span { class: cls, "{c.label}" }
        }
    });

    let theme_icon = if is_dark { IconKind::Sun } else { IconKind::Moon };

    rsx! {
        header { class: "topbar",
            div { class: "topbar__breadcrumbs", {crumbs_iter} }
            div { class: "topbar__search",
                Icon { kind: IconKind::Search, class: "topbar__search-icon".to_string() }
                input { id: "topbar-search", placeholder: "Search tables, partitions, rows…" }
                span { class: "kbd", "⌘K" }
            }
            div { class: "topbar__actions",
                button {
                    class: "topbar__icon-btn",
                    title: "Refresh",
                    onclick: move |_| on_refresh.call(()),
                    Icon { kind: IconKind::RefreshCw }
                }
                button {
                    class: "topbar__icon-btn",
                    title: "Toggle theme",
                    onclick: toggle_theme,
                    Icon { kind: theme_icon }
                }
            }
        }
    }
}
