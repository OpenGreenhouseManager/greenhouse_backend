use super::{Error, Result, schema::device};
use crate::Pool;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use greenhouse_core::device_service_dto::get_device::DeviceResponseDto;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, AsChangeset, Insertable)]
#[diesel(table_name = crate::database::schema::device)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct Device {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) address: String,
    pub(crate) description: String,
    pub(crate) canscript: bool,
    pub(crate) scraping: bool,
}

impl Device {
    pub(crate) fn new(
        name: &str,
        description: &str,
        device_address: &str,
        can_script: bool,
        scraping: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::from(name),
            description: String::from(description),
            address: String::from(device_address),
            canscript: can_script,
            scraping,
        }
    }

    pub(crate) async fn find_by_id(id: Uuid, pool: &Pool) -> Result<Self> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        device::table
            .filter(device::id.eq(id))
            .first(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::Find
            })
    }

    pub(crate) async fn all(pool: &Pool) -> Result<Vec<Self>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        device::table.get_results(&mut conn).await.map_err(|e| {
            sentry::capture_error(&e);
            Error::Find
        })
    }

    pub(crate) async fn flush(&mut self, pool: &Pool) -> Result<()> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        let db_entry = self.clone();
        diesel::insert_into(device::table)
            .values(&db_entry)
            .on_conflict(device::id)
            .do_update()
            .set(&db_entry)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::Creation
            })?;

        Ok(())
    }

    pub(crate) async fn get_scraping_devices(pool: &Pool) -> Result<Vec<Self>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        device::table
            .filter(device::scraping.eq(true))
            .get_results(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::Find
            })
    }
}

impl From<Device> for DeviceResponseDto {
    fn from(val: Device) -> Self {
        DeviceResponseDto {
            id: val.id.to_string(),
            name: val.name,
            address: val.address,
            description: val.description,
            canscript: val.canscript,
        }
    }
}
