// @generated automatically by Diesel CLI.

diesel::table! {
    push_subscription (id) {
        id -> Uuid,
        endpoint -> Text,
        p256dh -> Text,
        auth -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
