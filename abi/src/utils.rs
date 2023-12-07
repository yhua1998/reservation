use prost_types::Timestamp;
use sqlx::types::chrono::{DateTime, Utc};

pub fn convert_to_utc_time(ts: Timestamp) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as _).unwrap()
}

pub fn convert_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as _,
    }
}
