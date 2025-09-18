// @generated automatically by Diesel CLI.

diesel::table! {
    device (id) {
        id -> Uuid,
        name -> Varchar,
        address -> Varchar,
        description -> Varchar,
        canscript -> Bool,
        scraping -> Bool,
    }
}
