use super::{schema::diary_entry, Error, Result};
use crate::Pool;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Deserialize, AsChangeset, Insertable)]
#[diesel(table_name = crate::database::schema::diary_entry)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(remote = "DiaryEntry")]
pub struct DiaryEntry {
    #[serde(getter = "DiaryEntry::id")]
    id: Uuid,
    pub entry_date: chrono::NaiveDateTime,
    pub title: String,
    pub content: String,
    #[serde(getter = "DiaryEntry::created_at")]
    created_at: chrono::NaiveDateTime,
    #[serde(getter = "DiaryEntry::updated_at")]
    updated_at: chrono::NaiveDateTime,
}

impl DiaryEntry {
    pub fn new(entry_date: chrono::NaiveDateTime, title: &str, content: &str) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: Uuid::new_v4(),
            entry_date,
            title: String::from(title),
            content: String::from(content),
            created_at: now,
            updated_at: now,
        }
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

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn created_at(&self) -> chrono::NaiveDateTime {
        self.created_at
    }

    pub fn updated_at(&self) -> chrono::NaiveDateTime {
        self.updated_at
    }

    pub async fn flush(&mut self, pool: &Pool) -> Result<()> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        self.updated_at = chrono::Utc::now().naive_utc();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_diary_entry() {
        let entry_date = chrono::Utc::now().naive_utc();
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
        let entry_date = chrono::Utc::now().naive_utc();
        let title = "Test Title";
        let content = "Test Content";

        let entry1 = DiaryEntry::new(entry_date, title, content);
        let entry2 = DiaryEntry::new(entry_date, title, content);

        assert_ne!(entry1.id, entry2.id);
    }

    #[test]

    fn check_for_created_at_and_updated_at() {
        let entry_date = chrono::Utc::now().naive_utc();
        let title = "Test Title";
        let content = "Test Content";
        let entry = DiaryEntry::new(entry_date, title, content);

        assert_eq!(entry.created_at, entry.updated_at);
    }
}
