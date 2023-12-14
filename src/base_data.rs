use std::collections::HashMap;
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    pub static ref COUNTRIES: HashMap<String, BaseDataCountry> = {
        let vec: Vec<BaseDataCountry> = serde_yaml::from_str(include_str!("countries.yaml")).unwrap();
        vec.into_iter().map(|country| (country.name.to_lowercase(), country)).collect()
    };
}

#[derive(Deserialize, Clone)]
pub struct BaseDataCountry {
    pub name: String,
    pub iso_code: ISOCode,
}

#[derive(Deserialize, Clone)]
pub struct ISOCode {
    pub alpha2: String,
    pub alpha3: String,
}

impl BaseDataCountry {
    pub fn by_name(name: &str) -> Option<&'static BaseDataCountry> {
        COUNTRIES.get(&name.to_lowercase())
    }

    pub fn matches_filter(&self, filter: &str) -> bool {
        let filter = filter.to_lowercase();
        self.name.to_lowercase().contains(&filter)
            || self.iso_code.alpha2.to_lowercase().contains(&filter)
            || self.iso_code.alpha3.to_lowercase().contains(&filter)
    }
}
