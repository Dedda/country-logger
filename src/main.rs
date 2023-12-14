use clap::Parser;
use iced::{Application, Command, Element, Renderer, Subscription, widget::{column, row}};
use crate::svg_helper::COUNTRY_POLYGONS;
use crate::widgets::{CountryInfo, CountryInfoMessage, CountryList, CountryListMessage, WorldMap, WorldMapCountryFilter, WorldMapMessage};

mod base_data;
mod widgets;
mod database;
mod schema;
mod models;
mod svg_helper;

fn main() -> iced::Result {
    let _db_connection = database::connection().expect("Error opening database");
    println!("found {} svgs", COUNTRY_POLYGONS.iter().count());
    MyApp::run(iced::Settings {
        window: iced::window::Settings {
            size: (1000, 600),
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'd', long)]
    database_path: Option<String>,
}

struct MyApp {
    country_list: CountryList,
    country_info: Option<CountryInfo>,
    world_map: WorldMap,
}

#[derive(Debug)]
enum AppMessage {
    Event(iced::event::Event),
    CountryList(CountryListMessage),
    CountryInfo(CountryInfoMessage),
    WorldMap(WorldMapMessage),
}

impl From<CountryListMessage> for AppMessage {
    fn from(value: CountryListMessage) -> Self {
        AppMessage::CountryList(value)
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
            AppMessage::Event(_) => {}
            AppMessage::CountryList(msg) => {
                match &msg {
                    CountryListMessage::Search(_) => {}
                    CountryListMessage::Select(Some(country)) => {
                        self.country_info = Some(CountryInfo::new(country.clone()));
                        self.world_map.update(WorldMapMessage::FilterChanged(WorldMapCountryFilter::Include(vec![country.iso2.clone()])));
                    }
                    CountryListMessage::Select(None) => {
                        self.country_info = None;
                        self.world_map.update(WorldMapMessage::FilterRemoved);
                    }
                }
                self.country_list.update(msg);
            }
            AppMessage::CountryInfo(_) => {}
            AppMessage::WorldMap(msg) => self.world_map.update(msg),
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        row!(
            self.country_list.view().map(AppMessage::from),
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