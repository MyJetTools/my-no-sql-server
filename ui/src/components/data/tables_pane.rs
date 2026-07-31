use dioxus::prelude::*;
use std::collections::HashSet;

use crate::models::TableListItemApiModel;

#[component]
pub fn TablesPane(
    tables: Vec<TableListItemApiModel>,
    selected: String,
    writer_tables: HashSet<String>,
    partition_counts: std::collections::HashMap<String, usize>,
    on_select: EventHandler<String>,
) -> Element {
    let mut filter = use_signal(String::new);
    let filter_ra = filter.read();
    let needle = filter_ra.to_lowercase();
    let needle_empty = needle.is_empty();
    drop(filter_ra);

    let visible: Vec<TableListItemApiModel> = tables
        .into_iter()
        .filter(|t| needle_empty || t.name.to_lowercase().contains(&needle))
        .collect();

    let total = visible.len();

    let rows = visible.into_iter().map(|t| {
        let active = t.name == selected;
        let has_writer = writer_tables.contains(&t.name);
        let cls = if active {
            "tables-pane__item active"
        } else {
            "tables-pane__item"
        };
        let dot_cls = if has_writer {
            "tables-pane__dot has-writer"
        } else {
            "tables-pane__dot"
        };
        let part_count = partition_counts.get(&t.name).copied().unwrap_or(0);
        let part_str = super::format_compact_count(part_count);
        let name = t.name.clone();
        rsx! {
            div { class: cls, onclick: move |_| on_select.call(name.clone()),
                span { class: dot_cls }
                span { class: "tables-pane__name", "{t.name}" }
                span {
                    class: "tables-pane__count",
                    title: "Partitions",
                    "{part_str}"
                }
            }
        }
    });

    rsx! {
        aside { class: "tables-pane",
            div { class: "pane-header",
                span { class: "pane-header__title", "Tables" }
                span { class: "pane-header__count", "{total}" }
            }
            div { class: "pane-filter",
                input {
                    class: "filter-input",
                    placeholder: "filter tables…",
                    value: "{filter.read()}",
                    oninput: move |evt| filter.set(evt.value()),
                }
            }
            div { class: "pane-list", {rows} }
        }
    }
}
