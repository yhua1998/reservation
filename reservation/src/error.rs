use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("Database error")]
    DbError(#[from] sqlx::Error),
    #[error("Invalid start or end time for the reservation")]
    InvalidTime,
    #[error("unknown error")]
    Unknown,
}
