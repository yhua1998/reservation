mod error;
mod pb;
mod types;
mod utils;

pub use error::*;
pub use pb::*;
use std::fmt;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
enum RsvpStatus {
    Pending,
    Blocked,
    Confirmed,
    Unknown,
}

impl From<RsvpStatus> for ReservationStatus {
    fn from(status: RsvpStatus) -> Self {
        match status {
            RsvpStatus::Blocked => ReservationStatus::Blocked,
            RsvpStatus::Confirmed => ReservationStatus::Confirmed,
            RsvpStatus::Pending => ReservationStatus::Pending,
            RsvpStatus::Unknown => ReservationStatus::Unknown,
        }
    }
}

impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReservationStatus::Blocked => write!(f, "blocked"),
            ReservationStatus::Pending => write!(f, "pending"),
            ReservationStatus::Confirmed => write!(f, "confirmed"),
            ReservationStatus::Unknown => write!(f, "unknown"),
        }
    }
}
