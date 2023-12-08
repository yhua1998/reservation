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
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        // if reservation.status is pending, change to confirmed, otherwise do nothing.

        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;

        let sql = "UPDATE rsvp.reservations SET status = 'confirmed' WHERE id = $1 AND status = 'pending' RETURNING *";
        let reservation: abi::Reservation =
            sqlx::query_as(sql).bind(id).fetch_one(&self.pool).await?;
        Ok(reservation)
    }
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let sql = "UPDATE rsvp.reservations SET note = $1 WHERE id = $2 RETURNING *";
        let reservation: abi::Reservation = sqlx::query_as(sql)
            .bind(note)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(reservation)
    }

    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let sql = "SELECT * from rsvp.reservations WHERE id = $1";
        let rsvp = sqlx::query_as(sql).bind(id).fetch_one(&self.pool).await?;

        Ok(rsvp)
    }

    async fn delete(&self, id: ReservationId) -> Result<(), abi::Error> {
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let sql = "DELETE FROM rsvp.reservations WHERE id = $1";
        sqlx::query(sql).bind(id).execute(&self.pool).await?;
        Ok(())
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

    #[sqlx::test(migrations = "../migrations")]
    async fn reservation_change_status_should_work(migrated_pool: Pool<Postgres>) {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = abi::Reservation::new_pending(
            "tyrid",
            "ocean-view-room-714",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        let rsvp = manager.change_status(rsvp.id).await.unwrap();
        assert_eq!(rsvp.status, ReservationStatus::Confirmed as i32);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn reservation_change_status_not_pending_should_do_nothing(
        migrated_pool: Pool<Postgres>,
    ) {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = abi::Reservation::new_pending(
            "tyrid",
            "ocean-view-room-714",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        // change status agins should do nothing
        let err = manager.change_status(rsvp.id).await.unwrap_err();
        assert_eq!(err, abi::Error::NotFound);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn update_note_should_work(migrated_pool: Pool<Postgres>) {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = abi::Reservation::new_pending(
            "tyrid",
            "ocean-view-room-714",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        let rsvp = manager
            .update_note(rsvp.id, "I'll arrive at 4pm.".to_string())
            .await
            .unwrap();

        assert_eq!(rsvp.note, "I'll arrive at 4pm.");
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn get_reservation_should_work(migrated_pool: Pool<Postgres>) {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = abi::Reservation::new_pending(
            "tyrid",
            "ocean-view-room-714",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        let rsvp = manager.get(rsvp.id).await.unwrap();

        assert_eq!(rsvp.user_id, "tyrid");
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn delete_reservation_should_work(migrated_pool: Pool<Postgres>) {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = abi::Reservation::new_pending(
            "tyrid",
            "ocean-view-room-714",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        manager.delete(rsvp.id).await.unwrap();
    }
}
