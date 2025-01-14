use ::serde::{Deserialize, Serialize};
use ::time::{
    format_description::BorrowedFormatItem, macros::format_description, Duration, OffsetDateTime,
};

/// Type of time in engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineTime(String);

impl EngineTime {
    /// The format description of the time in engine.
    const FORMAT_DESC: &'static [BorrowedFormatItem<'static>] = format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]"
    );

    /// Return a new [EngineTime] with the current time.
    pub fn now() -> Self {
        Self(
            // Use `unwrap` because the format is fixed.
            OffsetDateTime::now_utc().format(Self::FORMAT_DESC).unwrap(),
        )
    }

    /// Get the elapsed time from the time of this [EngineTime].
    /// TODO: remove this if not used.
    pub fn elapsed_time(&self) -> Duration {
        let now = OffsetDateTime::now_utc();
        // Use `unwrap` because the format is fixed.
        let time = OffsetDateTime::parse(&self.0, Self::FORMAT_DESC).unwrap();
        now - time
    }
}
