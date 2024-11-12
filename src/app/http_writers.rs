use std::collections::BTreeMap;

use my_no_sql_server_core::rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct WriterInfo {
    pub version: String,
    pub last_ping: DateTimeAsMicroseconds,
    pub tables: Vec<String>,
}

pub struct HttpWriters {
    data: Mutex<BTreeMap<String, WriterInfo>>,
}

impl HttpWriters {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(BTreeMap::new()),
        }
    }

    pub async fn update(
        &self,
        name: &str,
        version: &str,
        tables: impl Iterator<Item = &str>,
        now: DateTimeAsMicroseconds,
    ) {
        let mut data = self.data.lock().await;
        match data.get_mut(name) {
            Some(writer_info) => {
                writer_info.last_ping = now;
                if writer_info.version != version {
                    writer_info.version = version.to_string();
                }

                writer_info.tables = tables.map(|x| x.to_string()).collect();
            }
            None => {
                data.insert(
                    name.to_string(),
                    WriterInfo {
                        version: version.to_string(),
                        last_ping: now,
                        tables: tables.map(|x| x.to_string()).collect(),
                    },
                );
            }
        }
    }

    pub async fn get<TResult>(
        &self,
        convert: impl Fn(&str, &WriterInfo) -> TResult,
    ) -> Vec<TResult> {
        let data = self.data.lock().await;

        let mut result = Vec::with_capacity(data.len());

        for (key, itm) in data.iter() {
            let itm = convert(key, itm);
            result.push(itm);
        }

        result
    }

    pub async fn gc(&self, now: DateTimeAsMicroseconds) {
        let mut data = self.data.lock().await;
        let mut to_remove = Vec::new();
        for (name, writer_info) in data.iter() {
            if now.duration_since(writer_info.last_ping).get_full_minutes() > 1 {
                to_remove.push(name.clone());
            }
        }

        for name in to_remove {
            data.remove(&name);
        }
    }
}
