use super::{
    Error, Result,
    diary_models::DiaryEntry,
    schema::{diary_entry, diary_entry_tag, diary_tag},
};
use crate::Pool;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::database::schema::diary_tag)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct DiaryTag {
    pub(crate) id: Uuid,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::database::schema::diary_entry_tag)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct DiaryEntryTag {
    pub(crate) diary_entry_id: Uuid,
    pub(crate) diary_tag_id: Uuid,
}

impl DiaryTag {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::from(name),
        }
    }

    pub(crate) async fn find_or_create(name: &str, pool: &Pool) -> Result<Self> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;

        let new_tag = Self::new(name);
        diesel::insert_into(diary_tag::table)
            .values(&new_tag)
            .on_conflict(diary_tag::name)
            .do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::Creation
            })?;

        diary_tag::table
            .filter(diary_tag::name.eq(name))
            .first(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::Find
            })
    }

    pub(crate) async fn get_tags_for_entry(entry_id: Uuid, pool: &Pool) -> Result<Vec<Self>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;

        diary_tag::table
            .inner_join(diary_entry_tag::table)
            .filter(diary_entry_tag::diary_entry_id.eq(entry_id))
            .select(diary_tag::all_columns)
            .load::<DiaryTag>(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::Find
            })
    }

    pub(crate) async fn find_entries_by_partial_name(
        partial: &str,
        pool: &Pool,
    ) -> Result<Vec<DiaryEntry>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;

        diary_entry::table
            .inner_join(diary_entry_tag::table.inner_join(diary_tag::table))
            .filter(diary_tag::name.ilike(format!("%{}%", partial)))
            .select(diary_entry::all_columns)
            .distinct()
            .load::<DiaryEntry>(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::Find
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_diary_tag() {
        let name = "test-tag";
        let tag = DiaryTag::new(name);
        assert_eq!(tag.name, name);
        assert!(!tag.id.is_nil());
    }

    #[test]
    fn test_diary_tag_id_uniqueness() {
        let tag1 = DiaryTag::new("tag-a");
        let tag2 = DiaryTag::new("tag-b");
        assert_ne!(tag1.id, tag2.id);
    }
}
