// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int8,
        email -> Varchar,
        password -> Bytea,
        first_name -> Nullable<Text>,
        last_name -> Nullable<Text>,
        created_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}
