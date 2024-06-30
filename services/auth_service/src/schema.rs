// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        password -> Text,
        salt -> Text,
        role -> Text,
    }
}
