use std::collections::VecDeque;

#[derive(Clone)]
pub struct MetricItem {
    pub status_code: u8,
    pub microseconds: i64,
}

#[derive(Clone)]
pub struct HttpMetricsByUrl {
    pub items: VecDeque<MetricItem>,
}

const MAX_ITEMS_COUNT: usize = 1024;

impl HttpMetricsByUrl {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    pub fn add(&mut self, status_code: u8, microseconds: i64) {
        if self.items.len() == MAX_ITEMS_COUNT {
            self.items.pop_front();
        }

        self.items.push_back(MetricItem {
            status_code,
            microseconds,
        })
    }
}
