// @generated automatically by Diesel CLI.

diesel::table! {
    user (id) {
        id -> Int8,
        email -> Nullable<Varchar>,
        first_name -> Nullable<Text>,
        last_name -> Nullable<Text>,
        password -> Nullable<Bytea>,
        created_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}
