use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

pub enum ActiveLoaderState<TLoader> {
    Active(Arc<TLoader>),
    Finished,
}

impl<TLoader> ActiveLoaderState<TLoader> {
    pub fn clone(&self) -> Self {
        match self {
            ActiveLoaderState::Active(state) => ActiveLoaderState::Active(state.clone()),
            ActiveLoaderState::Finished => ActiveLoaderState::Finished,
        }
    }
}

pub struct ActiveLoaders<TLoader> {
    loaders: Mutex<HashMap<String, ActiveLoaderState<TLoader>>>,
}

impl<TLoader> ActiveLoaders<TLoader> {
    pub fn new() -> Self {
        Self {
            loaders: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get(&self, table_name: &str) -> Option<ActiveLoaderState<TLoader>> {
        let access = self.loaders.lock().await;
        let result = access.get(table_name)?;

        let result = result.clone();

        Some(result)
    }

    async fn set(&self, table_name: &str, active_loader: ActiveLoaderState<TLoader>) {
        let mut access = self.loaders.lock().await;
        access.insert(table_name.to_string(), active_loader);
    }

    pub async fn set_as_finished(&self, table_name: &str) {
        self.set(table_name, ActiveLoaderState::Finished).await;
    }

    pub async fn set_loader(&self, table_name: &str, active_loader: Arc<TLoader>) {
        self.set(table_name, ActiveLoaderState::Active(active_loader))
            .await;
    }
}
