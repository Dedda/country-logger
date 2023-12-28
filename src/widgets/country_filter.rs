use iced::widget::{column, checkbox};

#[derive(Debug, Clone)]
pub enum CountryFiltersMessage {
    SearchString(String),
    OnlyVisited(bool),
}

pub struct CountryFilters {
    search_string: String,
    only_visited: bool,
}

impl CountryFilters {
    pub fn new() -> Self {
        Self {
            search_string: String::new(),
            only_visited: false,
        }
    }

    pub fn view(&self) -> iced::Element<'_, CountryFiltersMessage> {
        let search = iced::widget::text_input("Search", &self.search_string)
            .on_input(CountryFiltersMessage::SearchString);
        let only_visited = checkbox("only visited", self.only_visited, CountryFiltersMessage::OnlyVisited);
        column!(
            search,
            only_visited,
        )
        .width(iced::Length::Fixed(250.0))
        .into()
    }

    pub fn update(&mut self, msg: CountryFiltersMessage) {
        match msg {
            CountryFiltersMessage::SearchString(string) => self.search_string = string,
            CountryFiltersMessage::OnlyVisited(only_visited) => self.only_visited = only_visited,
        }
    }
}