use std::collections::HashMap;
use std::sync::Mutex;

struct IpCounter {
    intermediary: usize,
    value: usize,
}

// Per-IP request rate (requests per second). Incremented on every request,
// snapshotted once per second. Idle IPs are dropped on tick to keep the map
// bounded to currently-active clients.
pub struct RequestsPerIp {
    data: Mutex<HashMap<String, IpCounter>>,
}

impl RequestsPerIp {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }

    pub fn increase(&self, ip: &str) {
        let mut data = self.data.lock().unwrap();
        match data.get_mut(ip) {
            Some(counter) => counter.intermediary += 1,
            None => {
                data.insert(
                    ip.to_string(),
                    IpCounter {
                        intermediary: 1,
                        value: 0,
                    },
                );
            }
        }
    }

    pub fn one_second_tick(&self) {
        let mut data = self.data.lock().unwrap();
        data.retain(|_, counter| {
            counter.value = counter.intermediary;
            counter.intermediary = 0;
            counter.value != 0
        });
    }

    // Snapshot of currently-active connections: (addr, requests_per_second),
    // only those that had traffic in the last completed second.
    pub fn get_snapshot(&self) -> Vec<(String, usize)> {
        let data = self.data.lock().unwrap();
        data.iter()
            .filter(|(_, counter)| counter.value != 0)
            .map(|(addr, counter)| (addr.clone(), counter.value))
            .collect()
    }
}
