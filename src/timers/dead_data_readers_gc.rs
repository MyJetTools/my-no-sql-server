use std::{sync::Arc, time::Duration};

use crate::{
    app::{logs::SystemProcess, AppServices},
    date_time::MyDateTime,
};

pub async fn start(app: Arc<AppServices>) {
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

        let now = MyDateTime::utc_now();
        app.data_readers.gc(now, connection_inactive_duration).await;
    }
}
