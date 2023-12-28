// @generated automatically by Diesel CLI.

diesel::table! {
    countries (id) {
        id -> Integer,
        name -> Text,
        iso2 -> Text,
        iso3 -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    country_notes (id) {
        id -> Integer,
        country_id -> Integer,
        note -> Nullable<Text>,
        done -> Nullable<Bool>,
    }
}

diesel::table! {
    country_visits (id) {
        id -> Integer,
        country_id -> Integer,
    }
}

diesel::joinable!(country_notes -> countries (country_id));
diesel::joinable!(country_visits -> countries (country_id));

diesel::allow_tables_to_appear_in_same_query!(
    countries,
    country_notes,
    country_visits,
);
