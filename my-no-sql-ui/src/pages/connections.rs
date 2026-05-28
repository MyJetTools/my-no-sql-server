use std::time::Duration;

use dioxus::prelude::*;

use crate::api::get_connections;
use crate::models::{ConnectionReaderApiModel, ConnectionsApiModel};
use crate::utils::format_bytes;

const MAX_POINTS: usize = 120;

#[derive(Default)]
struct ConnectionsState {
    started: bool,
    snapshot: Option<ConnectionsApiModel>,
    history: Vec<(usize, usize)>,
}

impl ConnectionsState {
    fn push(&mut self, snapshot: ConnectionsApiModel) {
        self.history
            .push((snapshot.incoming_per_second, snapshot.outgoing_per_second));
        if self.history.len() > MAX_POINTS {
            let overflow = self.history.len() - MAX_POINTS;
            self.history.drain(0..overflow);
        }
        self.snapshot = Some(snapshot);
    }
}

#[component]
pub fn Connections() -> Element {
    let mut cs = use_signal(ConnectionsState::default);

    let started_val = cs.read().started;
    let on_mount = move |_| {
        if started_val {
            return;
        }
        cs.write().started = true;
        spawn(async move {
            loop {
                match get_connections().await {
                    Ok(result) => cs.write().push(result),
                    Err(err) => {
                        dioxus_utils::console_log(&format!("Connections error: {}", err));
                    }
                }
                dioxus_utils::js::sleep(Duration::from_secs(1)).await;
            }
        });
    };

    let cs_ra = cs.read();
    let history = cs_ra.history.clone();
    let snapshot = cs_ra.snapshot.clone();
    drop(cs_ra);

    let content = match snapshot {
        Some(snapshot) => render_connections(&history, &snapshot),
        None => rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "Connecting to server…" }
            }
        },
    };

    rsx! {
        section { class: "page page--padded", onmounted: on_mount,
            div { class: "connections", {content} }
        }
    }
}

fn render_connections(history: &[(usize, usize)], snapshot: &ConnectionsApiModel) -> Element {
    let incoming = snapshot.incoming_per_second;
    let outgoing = snapshot.outgoing_per_second;

    rsx! {
        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Traffic · all readers" }
                div { class: "conn-legend",
                    span { class: "conn-legend__item",
                        span { class: "conn-legend__dot conn-legend__dot--in" }
                        "Incoming "
                        b { "{format_bytes(incoming as f64)}/s" }
                    }
                    span { class: "conn-legend__item",
                        span { class: "conn-legend__dot conn-legend__dot--out" }
                        "Outgoing "
                        b { "{format_bytes(outgoing as f64)}/s" }
                    }
                }
            }
            div { class: "card__body",
                {render_chart(history)}
            }
        }
        {render_readers_table(&snapshot.readers)}
    }
}

fn render_chart(history: &[(usize, usize)]) -> Element {
    let width: f64 = 600.0;
    let height: f64 = 200.0;
    let pad: f64 = 10.0;

    let max = history
        .iter()
        .map(|(i, o)| (*i).max(*o))
        .max()
        .unwrap_or(0)
        .max(1) as f64;

    let usable_h = height - 2.0 * pad;
    let len = history.len();

    if len < 2 {
        return rsx! {
            div { class: "conn-chart conn-chart--empty",
                span { "Collecting data…" }
            }
        };
    }

    let step_x = width / (len - 1) as f64;

    let build_points = |selector: fn(&(usize, usize)) -> usize| -> String {
        history
            .iter()
            .enumerate()
            .map(|(idx, point)| {
                let x = idx as f64 * step_x;
                let value = selector(point) as f64;
                let y = pad + (1.0 - value / max) * usable_h;
                format!("{:.2},{:.2}", x, y)
            })
            .collect::<Vec<_>>()
            .join(" ")
    };

    let incoming_points = build_points(|(i, _)| *i);
    let outgoing_points = build_points(|(_, o)| *o);

    let max_label = format_bytes(max);

    rsx! {
        svg {
            class: "conn-chart",
            view_box: "0 0 {width} {height}",
            preserve_aspect_ratio: "none",
            line {
                class: "conn-chart__grid",
                x1: "0", y1: "{pad:.2}", x2: "{width}", y2: "{pad:.2}",
            }
            line {
                class: "conn-chart__grid",
                x1: "0", y1: "{height - pad:.2}", x2: "{width}", y2: "{height - pad:.2}",
            }
            polyline {
                class: "conn-chart__line conn-chart__line--in",
                points: "{incoming_points}",
                fill: "none",
            }
            polyline {
                class: "conn-chart__line conn-chart__line--out",
                points: "{outgoing_points}",
                fill: "none",
            }
            text {
                class: "conn-chart__label",
                x: "{width - 4.0}", y: "{pad + 10.0}",
                text_anchor: "end",
                "{max_label}/s"
            }
        }
    }
}

fn render_readers_table(readers: &[ConnectionReaderApiModel]) -> Element {
    if readers.is_empty() {
        return rsx! {
            div { class: "card",
                div { class: "card__body",
                    div { class: "empty-state",
                        div { class: "empty-state__title", "No active connections" }
                    }
                }
            }
        };
    }

    let rows = readers.iter().map(|reader| {
        let kind = if reader.is_node { "node" } else { "reader" };
        rsx! {
            tr {
                td { "{reader.name}" }
                td { "{reader.ip}" }
                td { class: "conn-table__kind", "{kind}" }
                td { class: "conn-table__num", "{format_bytes(reader.incoming_per_second as f64)}/s" }
                td { class: "conn-table__num", "{format_bytes(reader.outgoing_per_second as f64)}/s" }
                td { class: "conn-table__num", "{format_bytes(reader.pending_to_send as f64)}" }
            }
        }
    });

    rsx! {
        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Readers" }
                span { class: "card__subtitle", "{readers.len()} connected" }
            }
            div { class: "card__body",
                table { class: "conn-table",
                    thead {
                        tr {
                            th { "Name" }
                            th { "IP" }
                            th { "Kind" }
                            th { class: "conn-table__num", "Incoming/s" }
                            th { class: "conn-table__num", "Outgoing/s" }
                            th { class: "conn-table__num", "Pending" }
                        }
                    }
                    tbody { {rows} }
                }
            }
        }
    }
}
