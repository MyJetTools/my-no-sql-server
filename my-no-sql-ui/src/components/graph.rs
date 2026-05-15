use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum GraphFormat {
    Bytes,
    Duration,
}

impl GraphFormat {
    fn show(&self, value: f64) -> String {
        match self {
            GraphFormat::Bytes => crate::utils::format_bytes(value),
            GraphFormat::Duration => crate::utils::format_duration(value),
        }
    }
}

#[component]
pub fn Graph(values: Vec<f64>, format: GraphFormat) -> Element {
    let w = 50.0_f64;
    let height_attr = format!("{}", w as i64);

    let max = values.iter().cloned().fold(0.0_f64, f64::max);
    let coef = if max == 0.0 { 0.0 } else { w / max };
    let max_value = format.show(max);

    let bars = values.iter().enumerate().map(|(i, &m)| {
        let y = w - m * coef;
        rsx! {
            line {
                x1: "{i}",
                y1: "{w as i64}",
                x2: "{i}",
                y2: "{y}",
                style: "stroke:darkgray;stroke-width:2",
            }
        }
    });

    rsx! {
        svg { style: "font-size:16px", width: "240", height: "{height_attr}",
            rect {
                width: "240",
                height: "{height_attr}",
                style: "fill:none;stroke-width:;stroke:black",
            }
            {bars}
            text { x: "1", y: "16", fill: "black", "{max_value}" }
            text { x: "0", y: "15", fill: "lime", "{max_value}" }
        }
    }
}
