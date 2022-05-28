use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use rust_extensions::MyTimer;

use crate::{app::AppContext, background::persist::PersistTimer, db::DbTable};

use super::PersistType;

pub async fn spawn_dedicated_persist_thread(
    app: &Arc<AppContext>,
    db_table: Arc<DbTable>,
) -> Result<(), String> {
    let mut dedicated_thread = db_table.dedicated_thread.lock().await;

    let common_persist_thread = db_table.common_persist_thread.load(Ordering::SeqCst);
    if !common_persist_thread {
        return Err("Thread is already spawned".to_string());
    }

    db_table
        .common_persist_thread
        .store(false, Ordering::SeqCst);

    let mut timer = MyTimer::new(Duration::from_secs(1));

    let timer_name = format!("Persist: {}", db_table.name.as_str());
    timer.register_timer(
        timer_name.as_str(),
        Arc::new(PersistTimer::new(
            app.clone(),
            PersistType::Dedicated(db_table.clone()),
        )),
    );

    timer.start(app.clone(), app.clone());

    *dedicated_thread = Some(timer);

    Ok(())
}
