use iced::widget::{checkbox, column};
use crate::models::Country;

pub mod country_filter;
pub mod country_list;
pub mod world_map;

pub struct CountryInfo {
    country: Country,
    visited: bool,
}

#[derive(Debug)]
pub enum CountryInfoMessage {
    VisitCountry(Country),
    UnvisitCountry(Country),
}

impl CountryInfo {

    pub fn new(country: Country, visited: bool) -> Self {
        Self {
            country,
            visited,
        }
    }

    pub fn view(&self) -> iced::Element<'_, CountryInfoMessage> {
        let visited = checkbox("visited", self.visited, |visited| {
            if visited {
                CountryInfoMessage::VisitCountry(self.country.clone())
            } else {
                CountryInfoMessage::UnvisitCountry(self.country.clone())
            }
        });
        column!(
            iced::widget::text(&self.country.name)
            .size(25)
            .width(iced::Length::Fixed(200.0)),
            visited,
        ).into()
    }
}
