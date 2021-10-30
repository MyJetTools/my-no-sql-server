use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use super::ReaderSession;

pub struct SessionsListData {
    sessions: HashMap<u64, Arc<ReaderSession>>,
}

impl SessionsListData {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
}

pub struct SessionsList {
    data: RwLock<SessionsListData>,
}

impl SessionsList {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(SessionsListData::new()),
        }
    }

    pub async fn add(&self, session: Arc<ReaderSession>) {
        let mut write_lock = self.data.write().await;

        write_lock.sessions.insert(session.id, session);
    }

    pub async fn remove(&self, id: &u64) {
        let mut write_lock = self.data.write().await;
        write_lock.sessions.remove(id);
    }

    pub async fn get_all(&self) -> Vec<Arc<ReaderSession>> {
        let read_lock = self.data.read().await;
        read_lock.sessions.values().map(|itm| itm.clone()).collect()
    }

    pub async fn get_subscribed_to_table(
        &self,
        table_name: &str,
    ) -> Option<Vec<Arc<ReaderSession>>> {
        let mut result = None;

        let read_access = self.data.read().await;
        for session in read_access.sessions.values() {
            if session.has_table(table_name).await {
                if result.is_none() {
                    result = Some(Vec::new());
                }

                result.as_mut().unwrap().push(session.clone());
            }
        }

        result
    }
}
