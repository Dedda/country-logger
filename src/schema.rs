// @generated automatically by Diesel CLI.

diesel::table! {
    countries (id) {
        id -> Integer,
        name -> Text,
        iso2 -> Text,
        iso3 -> Text,
    }
}

diesel::table! {
    country_visits (id) {
        id -> Integer,
        country_id -> Integer,
    }
}

diesel::joinable!(country_visits -> countries (country_id));

diesel::allow_tables_to_appear_in_same_query!(
    countries,
    country_visits,
);
