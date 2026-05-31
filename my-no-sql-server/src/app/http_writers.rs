use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use my_no_sql_sdk::server::rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct WriterInfo {
    pub name: String,
    pub version: String,
    pub last_ping: DateTimeAsMicroseconds,
    pub tables: Vec<String>,
    pub addr: String,
}

// Writers are keyed by the `session` id issued during the Ping handshake, so
// two instances of the same app name are tracked separately and traffic can be
// attributed exactly (instead of guessing by IP).
pub struct HttpWriters {
    data: Mutex<BTreeMap<String, WriterInfo>>,
    next_session_id: AtomicU64,
}

impl HttpWriters {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(BTreeMap::new()),
            next_session_id: AtomicU64::new(0),
        }
    }

    fn generate_session_id(&self) -> String {
        let id = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        format!("Writer-{}", id)
    }

    // Refreshes the writer identified by `session` or creates a new entry,
    // returning the resulting session id. Resolution of the key:
    //   * `Some(s)` known        -> refresh that entry, return `s`.
    //   * `Some(_)` unknown      -> issue a fresh server `Writer-{n}` id (we do
    //                               not trust a client-claimed id we never gave,
    //                               e.g. after a restart/GC).
    //   * `None` (legacy client) -> stable synthetic `legacy:{name}` key so it
    //                               does not churn a new entry on every ping.
    pub async fn get_or_create(
        &self,
        session: Option<&str>,
        name: &str,
        version: &str,
        tables: impl Iterator<Item = &str>,
        addr: String,
        now: DateTimeAsMicroseconds,
    ) -> String {
        let mut data = self.data.lock().await;

        let session_id = match session {
            Some(session) if data.contains_key(session) => session.to_string(),
            Some(_) => self.generate_session_id(),
            None => format!("legacy:{}", name),
        };

        let writer_info = data.entry(session_id.clone()).or_insert_with(|| WriterInfo {
            name: name.to_string(),
            version: version.to_string(),
            last_ping: now,
            tables: Vec::new(),
            addr: addr.clone(),
        });

        writer_info.last_ping = now;
        writer_info.name = name.to_string();
        writer_info.version = version.to_string();
        writer_info.tables = tables.map(|x| x.to_string()).collect();
        writer_info.addr = addr;

        session_id
    }

    pub async fn get<TResult>(
        &self,
        convert: impl Fn(&str, &WriterInfo) -> TResult,
    ) -> Vec<TResult> {
        let data = self.data.lock().await;

        let mut result = Vec::with_capacity(data.len());

        for (session_id, itm) in data.iter() {
            let itm = convert(session_id, itm);
            result.push(itm);
        }

        result
    }

    pub async fn gc(&self, now: DateTimeAsMicroseconds) {
        let mut data = self.data.lock().await;
        let mut to_remove = Vec::new();
        for (session_id, writer_info) in data.iter() {
            if now.duration_since(writer_info.last_ping).get_full_minutes() > 1 {
                to_remove.push(session_id.clone());
            }
        }

        for session_id in to_remove {
            data.remove(&session_id);
        }
    }
}
