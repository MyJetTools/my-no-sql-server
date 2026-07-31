use std::collections::BTreeMap;
use std::sync::Arc;

use my_no_sql_sdk::core::db::DbNamespaceName;
use my_no_sql_sdk::server::DbInstance;

use crate::persist_markers::PersistMarkers;
use crate::persist_repo::PersistRepo;
use crate::settings_reader::SettingsModel;

/// Everything which belongs to a single namespace: its tables, its persistence
/// backend and its persist queue. Namespaces share nothing — a table of one
/// namespace is invisible to every other one, and each namespace persists into
/// a folder of its own (`<root>/default`, `<root>/alpha`, …).
pub struct DbNamespace {
    pub name: DbNamespaceName,
    pub db: DbInstance,
    pub repo: PersistRepo,
    pub persist_markers: PersistMarkers,
}

impl DbNamespace {
    pub async fn open(name: DbNamespaceName, settings: &SettingsModel) -> Self {
        let repo = settings.open_persist_repo(name.as_str()).await;

        Self {
            name,
            db: DbInstance::new(),
            repo,
            persist_markers: PersistMarkers::new(),
        }
    }

    pub fn tables_amount(&self) -> usize {
        self.db.get_tables().len()
    }
}

#[derive(Debug)]
pub enum DeleteNamespaceError {
    NotFound,
    IsDefault,
    /// The namespace still holds tables — deleting it would delete them too, so
    /// the caller has to empty it first.
    NotEmpty(usize),
}

/// All the namespaces of this server. A namespace materializes lazily: the first
/// write or the first subscription which mentions it brings it to life together
/// with its folder on disk.
pub struct DbNamespaces {
    /// `std::sync::RwLock` on purpose: a read only clones an `Arc` out of the map
    /// and the guard is never held across an `await`.
    items: std::sync::RwLock<BTreeMap<DbNamespaceName, Arc<DbNamespace>>>,
    /// Opening a persistence backend is async, so creation cannot happen under
    /// the map's lock. This one serializes it instead: without it two callers
    /// racing on the same new namespace would open its folder twice.
    create_lock: tokio::sync::Mutex<()>,
    settings: Arc<SettingsModel>,
}

impl DbNamespaces {
    pub fn new(settings: Arc<SettingsModel>) -> Self {
        Self {
            items: std::sync::RwLock::new(BTreeMap::new()),
            create_lock: tokio::sync::Mutex::new(()),
            settings,
        }
    }

    pub fn get(&self, name: &DbNamespaceName) -> Option<Arc<DbNamespace>> {
        self.items.read().unwrap().get(name).cloned()
    }

    pub fn get_default(&self) -> Arc<DbNamespace> {
        let name = DbNamespaceName::default();
        self.get(&name)
            .expect("Default namespace must be created during the app start up")
    }

    /// Returns the namespace, creating it (and its folder on disk) if this is the
    /// first time it is mentioned.
    pub async fn get_or_create(&self, name: &DbNamespaceName) -> Arc<DbNamespace> {
        if let Some(result) = self.get(name) {
            return result;
        }

        let _create_lock = self.create_lock.lock().await;

        // Somebody could have created it while we were waiting for the lock.
        if let Some(result) = self.get(name) {
            return result;
        }

        let namespace = Arc::new(DbNamespace::open(name.clone(), self.settings.as_ref()).await);

        self.items
            .write()
            .unwrap()
            .insert(name.clone(), namespace.clone());

        namespace
    }

    pub fn get_all(&self) -> Vec<Arc<DbNamespace>> {
        self.items.read().unwrap().values().cloned().collect()
    }

    /// Removes an empty namespace. The emptiness check happens under the same
    /// lock as the removal — otherwise a table created in between would be
    /// dropped from memory together with the namespace.
    pub fn delete(&self, name: &DbNamespaceName) -> Result<Arc<DbNamespace>, DeleteNamespaceError> {
        if name.is_default() {
            return Err(DeleteNamespaceError::IsDefault);
        }

        let mut write_access = self.items.write().unwrap();

        let namespace = match write_access.get(name) {
            Some(namespace) => namespace,
            None => return Err(DeleteNamespaceError::NotFound),
        };

        let tables_amount = namespace.tables_amount();

        if tables_amount > 0 {
            return Err(DeleteNamespaceError::NotEmpty(tables_amount));
        }

        Ok(write_access.remove(name).unwrap())
    }
}
