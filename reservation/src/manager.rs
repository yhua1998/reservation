use crate::{ReservationId, ReservationManager, Rsvp};
use abi::ReservationStatus;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, types::Uuid, PgPool, Row};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error> {
        rsvp.validate()?;

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan().into();

        let status: ReservationStatus = abi::ReservationStatus::try_from(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);

        // generate a insert sql for the reservation
        let id:Uuid = sqlx::query(
          "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id"
        ).bind(rsvp.user_id.clone())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(rsvp.note.clone())
        .bind(status.to_string())
        .fetch_one(&self.pool).await?.get(0);

        rsvp.id = id.to_string();
        Ok(rsvp)
    }
    async fn change_status(&self, _id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }
    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }
    async fn delete(&self, _id: ReservationId) -> Result<(), abi::Error> {
        todo!()
    }

    async fn get(&self, _id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }
    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
        todo!()
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use abi::ReservationConflictInfo;
    use sqlx::{Pool, Postgres};

    use super::*;

    #[sqlx::test(migrations = "../migrations")]
    async fn reserve_should_work_for_valid_window(migrated_pool: Pool<Postgres>) {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = abi::Reservation::new_pending(
            "tyrid",
            "ocean-view-room-713",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert!(!rsvp.id.is_empty());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn reserve_conflict_reservation_should_reject(pool: Pool<Postgres>) {
        let manager = ReservationManager::new(pool.clone());
        let rsvp1 = abi::Reservation::new_pending(
            "tyrid",
            "ocean-view-room-713",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "hello.",
        );
        let rsvp2 = abi::Reservation::new_pending(
            "aliceid",
            "ocean-view-room-713",
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            "hello.",
        );

        let _rsvp1 = manager.reserve(rsvp1).await.unwrap();
        let err = manager.reserve(rsvp2).await.unwrap_err();

        println!("{:?}", err);

        if let abi::Error::ConflictReservation(ReservationConflictInfo::Parsed(info)) = err {
            assert_eq!(info.old.rid, "ocean-view-room-713");
            assert_eq!(info.old.start.to_rfc3339(), "2022-12-25T22:00:00+00:00");
            assert_eq!(info.old.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
        } else {
            panic!("expect conflict reservation error");
        }
    }
}
