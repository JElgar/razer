// @generated automatically by Diesel CLI.

diesel::table! {
    my_models (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}
