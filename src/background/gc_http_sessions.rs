use std::{sync::Arc, time::Duration};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::app::{logs::SystemProcess, AppContext};

pub async fn start(app: Arc<AppContext>) {
    let delay = Duration::from_secs(10);

    while !app.states.is_shutting_down() {
        tokio::time::sleep(delay).await;

        let result = tokio::spawn(iteration(app.clone())).await;

        if let Err(err) = result {
            app.logs
                .add_fatal_error(
                    SystemProcess::Timer,
                    "gc_http_session".to_string(),
                    format!("Err:{}", err),
                )
                .await;
        }
    }
}

async fn iteration(app: Arc<AppContext>) {
    let now = DateTimeAsMicroseconds::now();

    for data_reader in app.data_readers.get_all().await {
        data_reader.ping_http_servers(now).await;
    }
    app.data_readers.ten_seconds_tick(now).await;
}
