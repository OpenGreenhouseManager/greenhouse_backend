use super::{schema::{diary_entry, diary_entry_alert}, Error, Result};
use crate::Pool;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use greenhouse_core::data_storage_service_dto::diary_dtos::get_diary_entry::DiaryEntryResponseDto;
use uuid::Uuid;
use crate::database::alert_models::Alert;

#[derive(Debug, Clone, Queryable, Selectable, AsChangeset, Insertable)]
#[diesel(table_name = crate::database::schema::diary_entry)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DiaryEntry {
    id: Uuid,
    pub entry_date: DateTime<Utc>,
    pub title: String,
    pub content: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Associations)]
#[diesel(belongs_to(DiaryEntry, foreign_key = diary_entry_id))]
#[diesel(belongs_to(Alert, foreign_key = alert_id))]
#[diesel(table_name = crate::database::schema::diary_entry_alert)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DiaryEntryAlert {
    pub diary_entry_id: Uuid,
    pub alert_id: Uuid,
}

impl DiaryEntry {
    pub fn new(entry_date: DateTime<Utc>, title: &str, content: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            entry_date,
            title: String::from(title),
            content: String::from(content),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub async fn find_by_id(id: Uuid, pool: &Pool) -> Result<Self> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        diary_entry::table
            .filter(diary_entry::id.eq(id))
            .first(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::FindError
            })
    }

    pub async fn find_by_date_range(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        pool: &Pool,
    ) -> Result<Vec<Self>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        diary_entry::table
            .filter(
                diary_entry::entry_date
                    .ge(start)
                    .and(diary_entry::entry_date.le(end)),
            )
            .load(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::FindError
            })
    }

    pub async fn flush(&mut self, pool: &Pool) -> Result<()> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        self.updated_at = chrono::Utc::now();
        let db_entry = self.clone();
        diesel::insert_into(diary_entry::table)
            .values(&db_entry)
            .on_conflict(diary_entry::id)
            .do_update()
            .set(&db_entry)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::CreationError
            })?;

        Ok(())
    }

    pub async fn delete(&self, pool: &Pool) -> Result<()> {
        let mut conn: bb8::PooledConnection<
            '_,
            diesel_async::pooled_connection::AsyncDieselConnectionManager<
                diesel_async::AsyncPgConnection,
            >,
        > = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        diesel::delete(diary_entry::table.filter(diary_entry::id.eq(self.id)))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::CreationError
            })?;

        Ok(())
    }

    pub async fn get_alerts(&self, pool: &Pool) -> Result<Vec<Alert>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        
        let alert_ids = diary_entry_alert::table
            .filter(diary_entry_alert::diary_entry_id.eq(self.id))
            .select(diary_entry_alert::alert_id)
            .load::<Uuid>(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::FindError
            })?;
            
        Alert::find_by_ids(&alert_ids, pool).await
    }

    pub async fn link_alert(&self, alert_id: Uuid, pool: &Pool) -> Result<()> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        
        let link = DiaryEntryAlert {
            diary_entry_id: self.id,
            alert_id,
        };
        
        diesel::insert_into(diary_entry_alert::table)
            .values(&link)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::CreationError
            })?;
            
        Ok(())
    }

    pub async fn unlink_alert(&self, alert_id: Uuid, pool: &Pool) -> Result<()> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        
        diesel::delete(
            diary_entry_alert::table
                .filter(diary_entry_alert::diary_entry_id.eq(self.id))
                .filter(diary_entry_alert::alert_id.eq(alert_id))
        )
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::DeletionError
        })?;
        
        Ok(())
    }

    pub async fn to_response_dto_with_alerts(&self, pool: &Pool) -> Result<DiaryEntryResponseDto> {
        let alerts = self.get_alerts(pool).await?;
        let mut dto = DiaryEntryResponseDto::from(self.clone());
        dto.alert_ids = alerts.iter().map(|alert| alert.id.to_string()).collect();
        Ok(dto)
    }
}

impl From<DiaryEntry> for DiaryEntryResponseDto {
    fn from(val: DiaryEntry) -> Self {
        DiaryEntryResponseDto {
            id: val.id.to_string(),
            date: val.entry_date.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
            title: val.title,
            content: val.content,
            created_at: val.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
            updated_at: val.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
            alert_ids: Vec::new(), // Empty by default, will be populated separately
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_diary_entry() {
        let entry_date = chrono::Utc::now();
        let title = "Test Title";
        let content = "Test Content";

        let entry = DiaryEntry::new(entry_date, title, content);

        assert_eq!(entry.entry_date, entry_date);
        assert_eq!(entry.title, title);
        assert_eq!(entry.content, content);
        assert_eq!(entry.created_at, entry.updated_at);
    }

    #[test]
    fn check_for_id_collision() {
        let entry_date = chrono::Utc::now();
        let title = "Test Title";
        let content = "Test Content";

        let entry1 = DiaryEntry::new(entry_date, title, content);
        let entry2 = DiaryEntry::new(entry_date, title, content);

        assert_ne!(entry1.id, entry2.id);
    }

    #[test]
    fn check_for_created_at_and_updated_at() {
        let entry_date = chrono::Utc::now();
        let title = "Test Title";
        let content = "Test Content";
        let entry = DiaryEntry::new(entry_date, title, content);

        assert_eq!(entry.created_at, entry.updated_at);
    }

    #[test]
    fn test_into_diary_entry_response_dto() {
        let entry_date = chrono::Utc::now();
        let title = "Test Title";
        let content = "Test Content";
        let created_at = chrono::Utc::now();
        let updated_at = chrono::Utc::now();
        let entry = DiaryEntry {
            id: Uuid::new_v4(),
            entry_date,
            title: String::from(title),
            content: String::from(content),
            created_at,
            updated_at,
        };

        let response: DiaryEntryResponseDto = entry.into();
        assert_ne!(response.id, "");
        assert_eq!(
            response.date,
            entry_date.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()
        );
        assert_eq!(response.title, title);
        assert_eq!(response.content, content);
        assert_eq!(
            response.created_at,
            created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()
        );
        assert_eq!(
            response.updated_at,
            updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()
        );
    }
}
