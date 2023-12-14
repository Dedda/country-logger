use diesel::{Insertable, Queryable, Selectable};
use crate::base_data::{BaseDataCountry, ISOCode};

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::countries)]
pub struct Country {
    pub id: i32,
    pub name: String,
    pub iso2: String,
    pub iso3: String,
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
}

impl From<BaseDataCountry> for NewCountry {
    fn from(value: BaseDataCountry) -> Self {
        let BaseDataCountry {
            name,
            iso_code: ISOCode {
                alpha2,
                alpha3,
            },
        } = value;
        Self {
            name,
            iso2: alpha2,
            iso3: alpha3,
        }
    }
}