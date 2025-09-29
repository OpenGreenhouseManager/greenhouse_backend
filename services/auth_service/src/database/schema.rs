// @generated automatically by Diesel CLI.

diesel::table! {
    preferences (id) {
        id -> Uuid,
        dashboard_preferences -> Jsonb,
        alert_preferences -> Jsonb,
        user_id -> Uuid,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        role -> Text,
        login_session -> Varchar,
        hash -> Varchar,
    }
}

diesel::joinable!(preferences -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(preferences, users,);
