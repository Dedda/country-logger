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
    country_borders (id) {
        id -> Nullable<Integer>,
        country_id -> Integer,
        polygon_data -> Text,
    }
}

diesel::table! {
    country_visits (id) {
        id -> Nullable<Integer>,
        country_id -> Nullable<Integer>,
    }
}

diesel::joinable!(country_borders -> countries (country_id));
diesel::joinable!(country_visits -> countries (country_id));

diesel::allow_tables_to_appear_in_same_query!(
    countries,
    country_borders,
    country_visits,
);
