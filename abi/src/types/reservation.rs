use std::ops::Bound;

use sqlx::{
    postgres::{types::PgRange, PgRow},
    types::{
        chrono::{DateTime, FixedOffset, Utc},
        Uuid,
    },
    FromRow, Row,
};

use crate::{utils::*, Error, Reservation, ReservationStatus, RsvpStatus, Validator};

use super::{get_timespan, validate_range};

impl Reservation {
    pub fn new_pending(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            id: "".to_string(),
            user_id: uid.into(),
            status: ReservationStatus::Pending as i32,
            resource_id: rid.into(),
            start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
            end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
            note: note.into(),
        }
    }

    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        get_timespan(self.start.as_ref(), self.end.as_ref())
    }
}

impl Validator for Reservation {
    fn validate(&self) -> Result<(), Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.clone()));
        }

        validate_range(self.start.as_ref(), self.end.as_ref())?;

        Ok(())
    }
}

impl FromRow<'_, PgRow> for Reservation {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = row.get("id");

        let range: PgRange<DateTime<Utc>> = row.get("timespan");
        let range: NativeRange<DateTime<Utc>> = range.into();

        let start = range.start.unwrap();
        let end = range.end.unwrap();

        let status: RsvpStatus = row.get("status");

        Ok(Self {
            id: id.to_string(),
            user_id: row.get("user_id"),
            resource_id: row.get("resource_id"),
            status: ReservationStatus::from(status) as i32,
            start: Some(convert_to_timestamp(start)),
            end: Some(convert_to_timestamp(end)),
            note: row.get("note"),
        })
    }
}

struct NativeRange<T> {
    start: Option<T>,
    end: Option<T>,
}

impl<T> From<PgRange<T>> for NativeRange<T> {
    fn from(range: PgRange<T>) -> Self {
        let f = |v: Bound<T>| match v {
            Bound::Included(v) => Some(v),
            Bound::Excluded(v) => Some(v),
            Bound::Unbounded => None,
        };
        let start = f(range.start);
        let end = f(range.end);
        Self { start, end }
    }
}
