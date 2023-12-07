mod error;
mod pb;
mod types;
mod utils;

pub use error::*;
pub use pb::*;
use std::fmt;

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
