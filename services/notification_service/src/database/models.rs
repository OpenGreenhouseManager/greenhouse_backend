use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, AsChangeset, Insertable)]
#[diesel(table_name = crate::database::schema::push_subscription)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct PushSubscription {
    pub(crate) id: Uuid,
    pub(crate) endpoint: String,
    pub(crate) p256dh: String,
    pub(crate) auth: String,
}