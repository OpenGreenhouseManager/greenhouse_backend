// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "severity"))]
    pub struct Severity;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Severity;

    alert (id) {
        id -> Uuid,
        severity -> Severity,
        identifier -> Uuid,
        value -> Text,
        note -> Nullable<Text>,
        created_at -> Timestamptz,
        datasource_id -> Uuid,
    }
}

diesel::table! {
    data_source (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        #[sql_name = "type"]
        #[max_length = 255]
        type_ -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    diary_entry (id) {
        id -> Uuid,
        entry_date -> Timestamptz,
        title -> Text,
        content -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(alert -> data_source (id));

diesel::allow_tables_to_appear_in_same_query!(
    alert,
    data_source,
    diary_entry,
);
