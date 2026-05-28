use dioxus::prelude::*;

use crate::components::atoms::{DeltaTone, Stat, StatTone, classify_reader, StateTone};
use crate::models::{ReaderApiModel, TableApiModel, WriterApiModel};
use crate::settings::HealthThresholds;
use crate::utils::format_mbit_per_sec;

#[component]
pub fn StatsRow(
    tables: Vec<TableApiModel>,
    writers: Vec<WriterApiModel>,
    readers: Vec<ReaderApiModel>,
    read_per_second: usize,
    write_payloads_per_second: usize,
) -> Element {
    let thresholds = *use_context::<Signal<HealthThresholds>>().read();
    let table_count = tables.len();
    let partitions: usize = tables.iter().map(|t| t.partitions_count).sum();
    let total_rows: usize = tables.iter().map(|t| t.records_amount).sum();
    let writer_count = writers.len();
    let writer_bindings: usize = writers.iter().map(|w| w.tables.len()).sum();

    let mut ok = 0;
    let mut warn = 0;
    let mut bad = 0;
    for r in readers.iter() {
        match classify_reader(&r.last_incoming_time, thresholds.warn_ms, thresholds.bad_ms) {
            StateTone::Ok => ok += 1,
            StateTone::Warn => warn += 1,
            StateTone::Bad => bad += 1,
            StateTone::Neutral => {}
        }
    }

    let reader_count = readers.len();
    let reader_tone = if bad > 0 {
        StatTone::Bad
    } else if warn > 0 {
        StatTone::Warn
    } else {
        StatTone::Ok
    };

    rsx! {
        div { class: "stats-row",
            Stat {
                label: "Tables".to_string(),
                value: format!("{table_count}"),
                delta: format!("{partitions} partitions"),
                tone: StatTone::Info,
            }
            Stat {
                label: "Rows in memory".to_string(),
                value: format_compact(total_rows),
                unit: "rows".to_string(),
                delta: format!("{} tables", table_count),
                tone: StatTone::Ok,
            }
            Stat {
                label: "Writers".to_string(),
                value: format!("{writer_count}"),
                unit: "connected".to_string(),
                delta: format!("{writer_bindings} table bindings"),
                tone: StatTone::Info,
            }
            Stat {
                label: "Readers".to_string(),
                value: format!("{reader_count}"),
                unit: "connected".to_string(),
                delta: format!("{ok} ok · {warn} slow · {bad} stalled"),
                delta_tone: tone_to_delta(reader_tone),
                tone: reader_tone,
            }
            Stat {
                label: "Read".to_string(),
                value: format_mbit_per_sec(read_per_second as f64),
                delta: "outgoing to readers".to_string(),
                tone: StatTone::Info,
            }
            Stat {
                label: "Write".to_string(),
                value: format!("{write_payloads_per_second}"),
                unit: "req/s".to_string(),
                delta: "post payloads".to_string(),
                tone: StatTone::Info,
            }
        }
    }
}

fn tone_to_delta(tone: StatTone) -> DeltaTone {
    match tone {
        StatTone::Ok => DeltaTone::Ok,
        StatTone::Warn => DeltaTone::Warn,
        StatTone::Bad => DeltaTone::Bad,
        StatTone::Info => DeltaTone::Neutral,
    }
}

pub fn format_compact(n: usize) -> String {
    let v = n as f64;
    if v >= 1_000_000_000.0 {
        format!("{:.1}B", v / 1_000_000_000.0)
    } else if v >= 1_000_000.0 {
        format!("{:.1}M", v / 1_000_000.0)
    } else if v >= 1_000.0 {
        format!("{:.1}K", v / 1_000.0)
    } else {
        format!("{}", n)
    }
}
