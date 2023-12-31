use diesel::{Associations, Insertable, Queryable, Selectable};
use crate::base_data::BaseDataCountry;

#[derive(Insertable, Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::countries)]
pub struct Country {
    pub id: i32,
    pub name: String,
    pub iso2: String,
    pub iso3: String,
    pub description: Option<String>,
}

impl Country {
    pub fn matches_filter(&self, filter: &str) -> bool {
        let filter = filter.to_lowercase();
        self.name.to_lowercase().contains(&filter)
            || self.iso2.to_lowercase().contains(&filter)
            || self.iso3.to_lowercase().contains(&filter)
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::countries)]
pub struct NewCountry {
    pub name: String,
    pub iso2: String,
    pub iso3: String,
    pub description: Option<String>,
}

impl From<BaseDataCountry> for NewCountry {
    fn from(value: BaseDataCountry) -> Self {
        Self {
            name: value.name,
            iso2: value.iso_code.alpha2,
            iso3: value.iso_code.alpha3,
            description: None,
        }
    }
}

#[derive(Insertable, Queryable, Selectable, Associations, Debug, Clone)]
#[diesel(belongs_to(Country))]
#[diesel(table_name = crate::schema::country_visits)]
pub struct CountryVisit {
    pub id: i32,
    pub country_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::country_visits)]
pub struct NewCountryVisit {
    pub country_id: i32,
}