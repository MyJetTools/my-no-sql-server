use std::{sync::Arc, time::Duration};

use crate::{app::AppServices, date_time::MyDateTime};

pub async fn start(app: Arc<AppServices>) {
    let delay = Duration::from_secs(5);
    let connection_inactive_duration = Duration::from_secs(60);
    loop {
        tokio::time::sleep(delay).await;

        let now = MyDateTime::utc_now();
        app.data_readers.gc(now, connection_inactive_duration).await;
    }
}
