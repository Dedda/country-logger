use std::path::Path;
use clap::Parser;
use Event::KeyReleased as KeyReleasedEvent;
use iced::{Application, Command, Element, Renderer, Subscription, widget::{column, row}};
use iced::event::Event::Keyboard as KeyboardEvent;
use iced::keyboard::{Event, KeyCode};
use crate::base_data::COUNTRIES;
use crate::database::{connection, is_country_visited, require_connection, unvisit_country, visit_country};
use crate::importer::simple_import;
use crate::svg_helper::COUNTRY_POLYGONS;
use crate::widgets::country_info::{CountryInfo, CountryInfoMessage};
use crate::widgets::country_filter::{CountryFilters, CountryFiltersMessage};
use crate::widgets::country_list::{CountryList, CountryListMessage};
use crate::widgets::world_map::{WorldMap, WorldMapCountryFilter, WorldMapMessage};

mod base_data;
mod widgets;
mod database;
mod schema;
mod models;
mod svg_helper;
mod importer;
mod flag_helper;

const ICON: &[u8] = include_bytes!("assets/globe_icon.png");

fn main() -> iced::Result {
    let _db_connection = require_connection();
    println!("found {} svgs for {} countries", COUNTRY_POLYGONS.iter().count(), COUNTRIES.len());
    if let Some(import_path) = Args::parse().simple_import {
        simple_import(Path::new(&import_path)).expect("Error during import");
    }
    if Args::parse().bootstrap_only {
        return Ok(())
    }
    let icon = iced::window::icon::from_file_data(ICON, Some(image::ImageFormat::Png)).expect("Cannot load icon");
    MyApp::run(iced::Settings {
        window: iced::window::Settings {
            size: (1200, 700),
            icon: Some(icon),
            ..Default::default()
        },
        antialiasing: true,
        ..Default::default()
    })
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'd', long)]
    database_path: Option<String>,
    #[arg(short = 'b')]
    bootstrap_only: bool,
    #[arg(short = 's', long = "simple-import")]
    simple_import: Option<String>,
}

struct MyApp {
    country_list: CountryList,
    country_filter: CountryFilters,
    country_info: Option<CountryInfo>,
    world_map: WorldMap,
}

#[derive(Debug)]
enum AppMessage {
    Event(iced::event::Event),
    CountryList(CountryListMessage),
    CountryFilter(CountryFiltersMessage),
    CountryInfo(CountryInfoMessage),
    WorldMap(WorldMapMessage),
}

impl From<CountryListMessage> for AppMessage {
    fn from(value: CountryListMessage) -> Self {
        AppMessage::CountryList(value)
    }
}

impl From<CountryFiltersMessage> for AppMessage {
    fn from(value: CountryFiltersMessage) -> Self {
        AppMessage::CountryFilter(value)
    }
}

impl From<CountryInfoMessage> for AppMessage {
    fn from(value: CountryInfoMessage) -> Self {
        AppMessage::CountryInfo(value)
    }
}

impl From<WorldMapMessage> for AppMessage {
    fn from(value: WorldMapMessage) -> Self {
        AppMessage::WorldMap(value)
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            country_list: CountryList::new(),
            country_filter: CountryFilters::new(),
            country_info: None,
            world_map: WorldMap::new(),
        }
    }
}

impl MyApp {
    fn view_map(&self) -> Element<'_, AppMessage> {
        self.world_map.view().map(AppMessage::from)
    }

    fn view_country_info(&self) -> Element<'_, AppMessage> {
        if let Some(info) = &self.country_info {
            info.view().map(AppMessage::from)
        } else {
            column!().into()
        }
    }

    fn update_iced_event(&mut self, event: iced::event::Event) {
        if let KeyboardEvent(KeyReleasedEvent { key_code, .. }) = event {
            if key_code == KeyCode::Escape && self.country_info.is_some() {
                self.country_info = None;
                self.world_map.update(WorldMapMessage::FilterRemoved);
            }
        }
    }

    fn update_country_list_event(&mut self, msg: CountryListMessage) {
        let mut connection = connection().expect("Cannot get database connection");
        match &msg {
            CountryListMessage::Search(_) => {}
            CountryListMessage::Select(Some(country)) => {
                self.country_info = Some(CountryInfo::new(country.clone(), is_country_visited(&mut connection, country).expect("Cannot determine country visits")));
                self.world_map.update(WorldMapMessage::FilterChanged(WorldMapCountryFilter::Include(vec![country.iso2.clone()])));
            }
            CountryListMessage::Select(None) => {
                self.country_info = None;
                self.world_map.update(WorldMapMessage::FilterRemoved);
            }
            CountryListMessage::FilterOnlyVisited(_) => {}
        }
        self.country_list.update(msg);
    }

    fn update_country_filter_event(&mut self, msg: CountryFiltersMessage) {
        match msg {
            CountryFiltersMessage::SearchString(search) => {
                self.country_list.update(CountryListMessage::Search(search.clone()));
                self.country_filter.update(CountryFiltersMessage::SearchString(search));
            }
            CountryFiltersMessage::OnlyVisited(only_visited) => {
                self.country_list.update(CountryListMessage::FilterOnlyVisited(only_visited));
                self.country_filter.update(CountryFiltersMessage::OnlyVisited(only_visited));
            }
        }
    }

    fn update_country_info_event(&mut self, msg: CountryInfoMessage) {
        let mut connection = connection().expect("Cannot get database connection");
        match &msg {
            CountryInfoMessage::VisitCountry(country) => {
                visit_country(&mut connection, country).expect("Cannot visit country");
            }
            CountryInfoMessage::UnvisitCountry(country) => {
                unvisit_country(&mut connection, country).expect("Cannot unvisit country");
            }
        }
        if let Some(country_info) = &mut self.country_info {
            country_info.update(msg);
        }
    }

}

impl Application for MyApp {
    type Executor = iced::executor::Default;
    type Message = AppMessage;
    type Theme = iced::theme::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let commands = vec![];
        (Default::default(), Command::batch(commands))
    }

    fn title(&self) -> String {
        "Country Logger".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessage::Event(event) => self.update_iced_event(event),
            AppMessage::CountryList(msg) => self.update_country_list_event(msg),
            AppMessage::CountryFilter(msg) => self.update_country_filter_event(msg),
            AppMessage::CountryInfo(msg) => self.update_country_info_event(msg),
            AppMessage::WorldMap(msg) => self.world_map.update(msg),
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let country_list = column!(
            self.country_filter.view().map(AppMessage::from),
            iced::widget::horizontal_rule(0),
            self.country_list.view().map(AppMessage::from),
        )
        .width(iced::Length::Fixed(250.0));
        row!(
            country_list,
            iced::widget::vertical_rule(0),
            self.view_map(),
            iced::widget::vertical_rule(0),
            self.view_country_info(),
        ).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let subs = vec![
            iced::subscription::events().map(AppMessage::Event)
        ];
        Subscription::batch(subs)
    }
}
