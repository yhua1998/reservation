mod error;
mod pb;
mod types;
mod utils;

pub use error::*;
pub use pb::*;

pub trait Validator {
    fn validate(&self) -> Result<(), Error>;
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
enum RsvpStatus {
    Pending,
    Blocked,
    Confirmed,
    Unknown,
}
