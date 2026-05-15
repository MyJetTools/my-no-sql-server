use dioxus::prelude::*;

use crate::components::atoms::{Badge, BadgeTone, Icon, IconKind};

#[component]
pub fn TableToolbar(
    filter_value: Signal<String>,
    writer_tags: Vec<String>,
    reader_count: usize,
) -> Element {
    let writer_pills = writer_tags.into_iter().map(|app| rsx! {
        Badge { text: app, tone: BadgeTone::Writer }
    });

    rsx! {
        div { class: "table-toolbar-new",
            input {
                class: "filter-input",
                placeholder: "filter rows… e.g. Status=\"ACTIVE\"",
                value: "{filter_value.read()}",
                oninput: move |evt| filter_value.set(evt.value()),
            }
            div { class: "table-toolbar-new__spacer" }
            div { class: "table-toolbar-new__group",
                {writer_pills}
            }
            div { class: "table-toolbar-new__group",
                Badge { text: format!("{reader_count} readers"), tone: BadgeTone::Reader }
            }
            button { class: "btn btn--sm",
                Icon { kind: IconKind::Download }
                "Export"
            }
            button { class: "btn btn--primary btn--sm",
                Icon { kind: IconKind::Plus }
                "New row"
            }
        }
    }
}
