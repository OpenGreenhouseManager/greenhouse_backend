// @generated automatically by Diesel CLI.

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
