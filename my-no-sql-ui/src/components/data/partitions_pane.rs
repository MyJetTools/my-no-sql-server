use dioxus::prelude::*;
use std::collections::HashMap;

#[component]
pub fn PartitionsPane(
    partitions: Vec<String>,
    counts: HashMap<String, usize>,
    selected: Option<String>,
    on_select: EventHandler<String>,
) -> Element {
    let mut filter = use_signal(String::new);
    let filter_ra = filter.read();
    let needle = filter_ra.to_lowercase();
    let needle_empty = needle.is_empty();
    drop(filter_ra);

    let visible: Vec<String> = partitions
        .into_iter()
        .filter(|p| needle_empty || p.to_lowercase().contains(&needle))
        .collect();

    let total = visible.len();

    let rows = visible.into_iter().map(|pk| {
        let active = selected.as_ref() == Some(&pk);
        let cls = if active {
            "partitions-pane__item active"
        } else {
            "partitions-pane__item"
        };
        let count = counts.get(&pk).copied().unwrap_or(0);
        let count_str = super::format_compact_count(count);
        let pk_for_handler = pk.clone();
        rsx! {
            div { class: cls, onclick: move |_| on_select.call(pk_for_handler.clone()),
                span { class: "partitions-pane__name", "{pk}" }
                span { class: "partitions-pane__count", "{count_str}" }
            }
        }
    });

    rsx! {
        aside { class: "partitions-pane",
            div { class: "pane-header",
                span { class: "pane-header__title", "Partitions · {total}" }
            }
            div { class: "pane-filter",
                input {
                    class: "filter-input",
                    placeholder: "filter partitions…",
                    value: "{filter.read()}",
                    oninput: move |evt| filter.set(evt.value()),
                }
            }
            div { class: "pane-list", {rows} }
        }
    }
}
