use jiff::{ToSpan, ZonedDateTime, civil::Time};

/// A ticker that fires daily at a given time.
pub struct DailyTicker {
    time: Time,
}
impl DailyTicker {
    pub fn new(time: Time) -> Self {
        Self { time }
    }

    pub async fn next(&self) {
        let now = ZonedDateTime::now();
        let mut target = now
            .with()
            .time(self.time)
            .build()
            .expect("failed to build target time???");
        // We're past today's event, wait until tomorrow.
        if target <= now {
            target = target.checked_add(1.day()).expect("date overflow???");
        }

        let delta = now.duration_until(&target);
        tokio::time::sleep(delta.try_into().expect("invalid duration???")).await
    }
}
