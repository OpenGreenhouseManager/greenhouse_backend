use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use crate::Pool;
use super::{schema, error::{Error, Result}};
use schema::push_subscription as push_subscription_table;

#[derive(Debug, Clone, Queryable, Selectable, AsChangeset, Insertable)]
#[diesel(table_name = crate::database::schema::push_subscription)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct PushSubscription {
    pub(crate) id: Uuid,
    pub(crate) endpoint: String,
    pub(crate) p256dh: String,
    pub(crate) auth: String,
}

impl PushSubscription {
	pub(crate) fn new(endpoint: String, p256dh: String, auth: String) -> Self {
		Self {
			id: Uuid::new_v4(),
			endpoint,
			p256dh,
			auth,
		}
	}

	pub(crate) async fn find_by_endpoint(endpoint: &str, pool: &Pool) -> Result<Option<Self>> {
		let mut conn = pool.get().await.map_err(|e| {
			sentry::capture_error(&e);
			Error::DatabaseConnection
		})?;

		let result: core::result::Result<Self, diesel::result::Error> = push_subscription_table::table
			.filter(push_subscription_table::endpoint.eq(endpoint))
			.select((
				push_subscription_table::id,
				push_subscription_table::endpoint,
				push_subscription_table::p256dh,
				push_subscription_table::auth,
			))
			.first::<Self>(&mut conn)
			.await;

		match result {
			Ok(record) => Ok(Some(record)),
			Err(diesel::result::Error::NotFound) => Ok(None),
			Err(e) => {
				sentry::capture_error(&e);
				Err(Error::Find)
			}
		}
	}

	pub(crate) async fn all(pool: &Pool) -> Result<Vec<Self>> {
		let mut conn = pool.get().await.map_err(|e| {
			sentry::capture_error(&e);
			Error::DatabaseConnection
		})?;
		push_subscription_table::table
			.select((
				push_subscription_table::id,
				push_subscription_table::endpoint,
				push_subscription_table::p256dh,
				push_subscription_table::auth,
			))
			.get_results::<Self>(&mut conn)
			.await
			.map_err(|e| {
				sentry::capture_error(&e);
				Error::Find
			})
	}

	pub(crate) async fn flush(&self, pool: &Pool) -> Result<()> {
		let mut conn = pool.get().await.map_err(|e| {
			sentry::capture_error(&e);
			Error::DatabaseConnection
		})?;
		let db_entry = self.clone();
		diesel::insert_into(push_subscription_table::table)
			.values(&db_entry)
			.on_conflict(push_subscription_table::id)
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

	pub(crate) async fn update_keys(&mut self, p256dh: String, auth: String, pool: &Pool) -> Result<()> {
		self.p256dh = p256dh;
		self.auth = auth;
		self.flush(pool).await
	}
}