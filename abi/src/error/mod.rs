use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

mod conflict;

pub use conflict::ReservationConflictInfo;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error")]
    DbError(sqlx::Error),

    #[error("No reservation found by the given condition")]
    NotFound,

    #[error("Invalid start or end time for the reservation")]
    InvalidTime,

    #[error("Conflict reservation")]
    ConflictReservation(ReservationConflictInfo),

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid reservation id {0}")]
    InvalidReservationId(String),

    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),

    #[error("unknown error")]
    Unknown,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DbError(_), Self::DbError(_)) => true,
            (Self::ConflictReservation(l0), Self::ConflictReservation(r0)) => l0 == r0,
            (Self::InvalidUserId(l0), Self::InvalidUserId(r0)) => l0 == r0,
            (Self::InvalidReservationId(l0), Self::InvalidReservationId(r0)) => l0 == r0,
            (Self::InvalidResourceId(l0), Self::InvalidResourceId(r0)) => l0 == r0,
            (Self::NotFound, Self::NotFound) => true,
            _ => false,
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Error::ConflictReservation(err.detail().unwrap().parse().unwrap())
                    }
                    _ => Error::DbError(sqlx::Error::Database(e)),
                }
            }
            sqlx::Error::RowNotFound => Error::NotFound,
            _ => Error::DbError(e),
        }
    }
}
