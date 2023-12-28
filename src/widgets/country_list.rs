use iced::widget::{row, column};
use iced::widget::image::Handle;
use itertools::Itertools;
use crate::database;
use crate::flag_helper::FLAGS;
use crate::models::Country;

pub struct CountryList {
    filter: String,
    filter_only_visited: bool,
}

#[derive(Debug, Clone)]
pub enum CountryListMessage {
    Search(String),
    FilterOnlyVisited(bool),
    Select(Option<Country>),
}

impl CountryList {
    pub fn new() -> Self {
        Self {
            filter: String::new(),
            filter_only_visited: false,
        }
    }

    #[allow(unstable_name_collisions)]
    pub fn view(&self) -> iced::Element<'_, CountryListMessage> {
        let countries = self.get_filtered_countries().iter()
            .map(|country| {
                let null_flag = &Handle::from_memory([]);
                let flag = FLAGS.get(&country.iso2).unwrap_or(null_flag);
                let flag = iced::widget::image(flag.clone())
                    .width(iced::Length::Fixed(40.0));
                iced::widget::button(row!(
                    iced::widget::text(&country.name),
                    iced::widget::horizontal_space(iced::Length::Fill),
                    flag,
                    iced::widget::horizontal_space(iced::Length::Fixed(10.0)),
                ))
                    .on_press(CountryListMessage::Select(Some(country.clone())))
                    .width(iced::Length::Fill)
                    .style(iced::theme::Button::Text)
                    .into()
            })
            .intersperse_with(|| {
                iced::widget::horizontal_rule(0).into()
            })
            .collect::<Vec<iced::Element<'_, CountryListMessage>>>();
        let countries = iced::widget::column(countries);
        let countries_scrollable = iced::widget::scrollable(countries)
            .direction(iced::widget::scrollable::Direction::Vertical(Default::default()));
        column!(
            iced::widget::vertical_space(10),
            countries_scrollable,
        )
            .height(iced::Length::Fill)
            .width(iced::Length::Fixed(250.0))
            .into()
    }

    pub fn update(&mut self, message: CountryListMessage) {
        match message {
            CountryListMessage::Search(filter) => {
                self.filter = filter;
            }
            CountryListMessage::Select(_) => {}
            CountryListMessage::FilterOnlyVisited(only_visited) => {
                self.filter_only_visited = only_visited;
            }
        }
    }

    fn get_filtered_countries(&self) -> Vec<Country> {
        database::all_countries_with_visit_status(&mut database::connection().unwrap()).unwrap().into_iter()
            .filter(|(country, _visited)| country.matches_filter(&self.filter.to_lowercase()))
            .filter(|(_country, visited)| !self.filter_only_visited || *visited)
            .map(|(country, _visited)| country)
            .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
            .collect()
    }
}
