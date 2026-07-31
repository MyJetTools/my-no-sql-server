use std::{sync::Arc, time::Duration};

use crate::app::AppContext;

pub async fn shutdown(app: &Arc<AppContext>) {
    println!("Doing persistence before shut down");
    let duration = Duration::from_secs(1);
    tokio::time::sleep(duration).await;

    while has_something_to_persist(app).await {
        println!("Has something to persist. Persisting and checking again...");
        crate::operations::persist::persist(app).await;
        tokio::time::sleep(duration).await;
    }

    println!("Everthing is persisted. App can be closed now");
}

async fn has_something_to_persist(app: &Arc<AppContext>) -> bool {
    for db_namespace in app.namespaces.get_all() {
        if db_namespace
            .persist_markers
            .has_something_to_persist()
            .await
        {
            return true;
        }
    }

    false
}
