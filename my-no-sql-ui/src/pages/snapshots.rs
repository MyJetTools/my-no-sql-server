use dioxus::prelude::*;

use crate::api;

#[derive(Default)]
struct SnapshotsState {
    started: bool,
    loading: bool,
    files: Vec<String>,
    error: Option<String>,
}

#[component]
pub fn Snapshots() -> Element {
    let mut cs = use_signal(SnapshotsState::default);

    let mut load = move || {
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

    let on_mount = move |_| {
        if cs.read().started {
            return;
        }
        cs.write().started = true;
        load();
    };

    let cs_ra = cs.read();
    let loading = cs_ra.loading;
    let files = cs_ra.files.clone();
    let error = cs_ra.error.clone();
    drop(cs_ra);

    let body = if let Some(err) = error.clone() {
        rsx! {
            div { style: "color: var(--danger); font-size: 12.5px;", "{err}" }
        }
    } else if loading && files.is_empty() {
        rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "Loading snapshots…" }
            }
        }
    } else if files.is_empty() {
        rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "No snapshots yet" }
                div { class: "empty-state__sub",
                    "Snapshots are written periodically into the backup folder configured on the server."
                }
            }
        }
    } else {
        let count = files.len();
        rsx! {
            div { class: "card",
                div { class: "card__header",
                    span { class: "card__title", "Snapshot files" }
                    span { class: "card__subtitle", "{count} file(s)" }
                }
                div { class: "card__body",
                    table { class: "rows-table",
                        thead {
                            tr {
                                th { "File name" }
                            }
                        }
                        tbody {
                            for name in files.iter() {
                                tr { key: "{name}",
                                    td { style: "font-family: var(--font-mono);", "{name}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    rsx! {
        section { class: "page page--padded", onmounted: on_mount,
            div { style: "max-width: 720px; display: flex; flex-direction: column; gap: 14px;",
                div { style: "display: flex; justify-content: flex-end;",
                    button {
                        class: "btn btn--ghost btn--sm",
                        disabled: loading,
                        onclick: move |_| load(),
                        if loading { "Refreshing…" } else { "Refresh" }
                    }
                }
                {body}
            }
        }
    }
}
