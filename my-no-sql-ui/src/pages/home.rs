use std::time::Duration;

use dioxus::prelude::*;

use crate::api::get_status;
use crate::components::{ConnectedStatusBar, DisconnectedStatusBar, Graph, GraphFormat};
use crate::models::{
    ReaderApiModel, StatusApiModel, TableApiModel, WriterApiModel,
};
use crate::utils::format_unix_microseconds;

#[component]
pub fn Home() -> Element {
    let mut data = use_signal(|| None::<StatusApiModel>);
    let mut started = use_signal(|| false);

    let started_val = *started.read();
    let on_mount = move |_| {
        if started_val {
            return;
        }
        *started.write() = true;
        spawn(async move {
            loop {
                match get_status().await {
                    Ok(result) => {
                        data.set(Some(result));
                    }
                    Err(err) => {
                        dioxus_utils::console_log(&format!("Status error: {}", err));
                        data.set(None);
                    }
                }
                dioxus_utils::js::sleep(Duration::from_secs(1)).await;
            }
        });
    };

    let snapshot = data.read().clone();

    let content = match &snapshot {
        Some(s) => {
            if let Some(initialized) = s.initialized.as_ref() {
                let nodes: Vec<ReaderApiModel> = initialized
                    .readers
                    .iter()
                    .filter(|r| r.is_node)
                    .cloned()
                    .collect();
                let readers: Vec<ReaderApiModel> = initialized
                    .readers
                    .iter()
                    .filter(|r| !r.is_node)
                    .cloned()
                    .collect();

                rsx! {
                    h3 { "Connected nodes" }
                    {render_readers(nodes)}
                    h3 { "Writers" }
                    {render_writers(initialized.writers.clone())}
                    h3 { "Readers" }
                    {render_readers(readers)}
                    h3 { "Tables" }
                    {render_tables(initialized.tables.clone())}
                    ConnectedStatusBar { data: s.status_bar.clone() }
                }
            } else {
                rsx! {
                    h3 { "Server is not initialized" }
                    ConnectedStatusBar { data: s.status_bar.clone() }
                }
            }
        }
        None => rsx! {
            div { style: "padding:10px;", "Loading..." }
            DisconnectedStatusBar {}
        },
    };

    rsx! {
        section { onmounted: on_mount, {content} }
    }
}

fn render_writers(writers: Vec<WriterApiModel>) -> Element {
    let rows = writers.into_iter().map(|w| {
        let tables = w.tables.iter().map(|t| {
            rsx! {
                span { class: "badge text-bg-success", "{t}" }
            }
        });
        rsx! {
            tr {
                td { "{w.name}" }
                td { "{w.version}" }
                td { {tables} }
                td { "{w.last_update}" }
            }
        }
    });

    rsx! {
        table { class: "table table-striped",
            thead {
                tr {
                    th { "App" }
                    th { "Version" }
                    th { "Tables" }
                    th { "Last ping" }
                }
            }
            tbody { {rows} }
        }
    }
}

fn render_readers(readers: Vec<ReaderApiModel>) -> Element {
    let rows = readers.into_iter().map(|r| {
        let sent: Vec<f64> = r.sent_per_second.iter().map(|v| *v as f64).collect();
        let tables = r.tables.iter().map(|t| {
            rsx! {
                span { class: "badge text-bg-primary", "{t}" }
            }
        });
        rsx! {
            tr {
                td { "{r.id}" }
                td { "{r.name}" }
                td {
                    div { "{r.ip}" }
                    Graph { values: sent, format: GraphFormat::Bytes }
                }
                td { {tables} }
                td {
                    div {
                        b { "Connected" }
                        ": {r.connected_time}"
                    }
                    div {
                        b { "Incoming" }
                        ": {r.last_incoming_time}"
                    }
                    div {
                        b { "ToSend" }
                        ": {r.pending_to_send}"
                    }
                }
            }
        }
    });

    rsx! {
        table { class: "table table-striped",
            thead {
                tr {
                    th { "Id" }
                    th { "Client" }
                    th { "Ip" }
                    th { "Tables" }
                    th { "Stats" }
                }
            }
            tbody { {rows} }
        }
    }
}

fn render_tables(tables: Vec<TableApiModel>) -> Element {
    let mut total_partitions: usize = 0;
    let mut total_records: usize = 0;
    let mut total_data_size: usize = 0;
    let total_indexed_records: usize = 0;

    let rows: Vec<Element> = tables
        .into_iter()
        .map(|t| {
            let max_partitions = t
                .max_partitions_amount
                .map(|v| v.to_string())
                .unwrap_or_else(|| "∞".to_string());
            let max_rows = t
                .max_rows_per_partition
                .map(|v| v.to_string())
                .unwrap_or_else(|| "∞".to_string());

            let last_update = format_unix_microseconds(t.last_update_time);
            let last_persist = t
                .last_persist_time
                .map(format_unix_microseconds)
                .unwrap_or_else(|| "----".to_string());
            let next_persist = t
                .next_persist_time
                .map(format_unix_microseconds)
                .unwrap_or_else(|| "---".to_string());

            total_partitions += t.partitions_count;
            total_records += t.records_amount;
            total_data_size += t.data_size;

            let persist_badge = if t.persist {
                rsx! {
                    span { class: "badge text-bg-success", "Persist" }
                }
            } else {
                rsx! {
                    span { class: "badge text-bg-warning", "Not Persist" }
                }
            };

            let persist_duration: Vec<f64> =
                t.last_persist_duration.iter().map(|v| *v as f64).collect();

            rsx! {
                tr {
                    td {
                        div { "{t.name}" }
                        div {
                            {persist_badge}
                            div {
                                span { class: "badge text-bg-primary", "Max partitions: {max_partitions}" }
                            }
                            div {
                                span { class: "badge text-bg-primary", "Max rows per partition: {max_rows}" }
                            }
                        }
                    }
                    td { "{t.persist_amount}" }
                    td { "{t.data_size}" }
                    td { "{t.avg_entity_size}" }
                    td { "{t.partitions_count}" }
                    td { "{t.records_amount}" }
                    td { "{t.expiration_index_records_amount}" }
                    td {
                        div { "UpdateTime: {last_update}" }
                        div { "PersistTime: {last_persist}" }
                        div { "NextPersist: {next_persist}" }
                        div { Graph { values: persist_duration, format: GraphFormat::Duration } }
                    }
                }
            }
        })
        .collect();

    rsx! {
        table { class: "table table-striped",
            thead {
                tr {
                    th { "Table" }
                    th { "Persist" }
                    th { "DataSize" }
                    th { "Avg entity size" }
                    th { "Partitions" }
                    th { "Records" }
                    th { "Indexed Records" }
                    th { "Last update" }
                }
            }
            tbody {
                {rows.into_iter()}
                tr { style: "font-weight: bold; background-color:black;",
                    td { style: "color:white;", "Total" }
                    td {}
                    td { style: "color:white;", "DataSize: {total_data_size}" }
                    td {}
                    td { style: "color:white;", "Partitions: {total_partitions}" }
                    td { style: "color:white;", "Records: {total_records}" }
                    td { style: "color:white;", "Indexed: {total_indexed_records}" }
                    td {}
                }
            }
        }
    }
}
