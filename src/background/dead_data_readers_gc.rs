use std::{sync::Arc, time::Duration};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::app::{logs::SystemProcess, AppContext};

pub async fn start(app: Arc<AppContext>) {
    let delay = Duration::from_secs(5);
    let connection_inactive_duration = Duration::from_secs(60);
    app.logs
        .add_info(
            None,
            SystemProcess::System,
            "Timer dead data readers gc".to_string(),
            "Started".to_string(),
        )
        .await;
    loop {
        tokio::time::sleep(delay).await;

        let now = DateTimeAsMicroseconds::now();

        let sessions = app.data_readers.get_all().await;

        for session in sessions {
            let incoming_trafic_moment = session.metrics.get_incoming_traffic_moment().await;

            if now.duration_since(incoming_trafic_moment) > connection_inactive_duration {
                crate::operations::sessions::disconnect(session).await;
            }
        }
    }
}
