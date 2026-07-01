use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};

use my_no_sql_sdk::core::rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

use crate::app::AppContext;

/// How often the persistence backend is compacted.
const VACUUM_INTERVAL_SECS: u64 = 60 * 60;

/// Wakes up every minute and compacts the persistence backend once an hour has
/// passed since the previous run (SQLite `VACUUM`, or dropping fully-freed
/// page-files for the Files backend). The last-run timestamp is kept in memory,
/// so after a restart the first vacuum happens an hour later.
pub struct VacuumTimer {
    app: Arc<AppContext>,
    last_vacuum_unix_micros: AtomicI64,
}

impl VacuumTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self {
            app,
            last_vacuum_unix_micros: AtomicI64::new(
                DateTimeAsMicroseconds::now().unix_microseconds,
            ),
        }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for VacuumTimer {
    async fn tick(&self) {
        let now = DateTimeAsMicroseconds::now();
        let last_vacuum =
            DateTimeAsMicroseconds::new(self.last_vacuum_unix_micros.load(Ordering::Relaxed));

        if now
            .duration_since(last_vacuum)
            .as_positive_or_zero()
            .as_secs()
            < VACUUM_INTERVAL_SECS
        {
            return;
        }

        println!("Running persistence vacuum...");
        self.app.repo.vacuum().await;
        self.last_vacuum_unix_micros
            .store(now.unix_microseconds, Ordering::Relaxed);
        println!("Persistence vacuum completed");
    }
}
