use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::app::AppContext;

pub async fn start(app: Arc<AppContext>, mut receiver: UnboundedReceiver<()>) {
    loop {
        receiver.recv().await;
        handle_transactions(app.as_ref()).await;
    }
}

#[inline]
async fn handle_transactions(app: &AppContext) {
    let next_events = app.events_dispatcher.get_next_events().await;

    if next_events.is_none() {
        return;
    }

    let next_events = next_events.unwrap();
    super::to_readers::broadcast(app, &next_events).await;

    let db_table = app.db.get_table(next_events.table_name.as_str()).await;
    super::persist::execute(db_table, &app.updates_to_persist_by_table, &next_events).await;
}
