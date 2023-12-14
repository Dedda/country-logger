use iced::mouse::Cursor;
use iced::{Color, Rectangle};
use iced::widget::{canvas, column, row};
use iced::widget::canvas::{Fill, fill, Frame, Path, Stroke, stroke};
use itertools::Itertools;
use crate::database;
use crate::models::Country;
use crate::svg_helper::{COUNTRY_POLYGONS, Point, Polygon};

pub struct CountryList {
    filter: String,
}

#[derive(Debug, Clone)]
pub enum CountryListMessage {
    Search(String),
    Select(Option<Country>),
}

impl CountryList {

    pub fn new() -> Self {
        Self {
            filter: String::new(),
        }
    }

    #[allow(unstable_name_collisions)]
    pub fn view(&self) -> iced::Element<'_, CountryListMessage> {
        let search = iced::widget::text_input("Search", &self.filter)
            .on_input(CountryListMessage::Search);
        let countries = Self::get_filtered_countries(self.filter.to_lowercase()).iter()
            .map(|country| iced::widget::button(row!(
                iced::widget::text(&country.name),
                iced::widget::horizontal_space(iced::Length::Fill),
                iced::widget::text(format!("[{}]", country.iso2)),
                iced::widget::horizontal_space(iced::Length::Fixed(10.0)),
            ))
                .on_press(CountryListMessage::Select(Some(country.clone())))
                .width(iced::Length::Fill)
                .style(iced::theme::Button::Text)
                .into())
            .intersperse_with(|| {
                iced::widget::horizontal_rule(0).into()
            })
            .collect::<Vec<iced::Element<'_, CountryListMessage>>>();
        let countries = iced::widget::column(countries);
        let countries_scrollable = iced::widget::scrollable(countries)
            .direction(iced::widget::scrollable::Direction::Vertical(Default::default()));
        column!(
            search,
            iced::widget::vertical_space(10),
            countries_scrollable,
        )
            .height(iced::Length::Fill)
            .width(iced::Length::Fixed(250.0))
            .into()
    }

    pub fn update(&mut self, message: CountryListMessage) -> Vec<CountryListMessage> {
        match message {
            CountryListMessage::Search(filter) => {
                self.filter = filter;
            }
            CountryListMessage::Select(_) => {}
        }
        vec![]
    }

    fn get_filtered_countries(filter: String) -> Vec<Country> {
        database::all_countries(&mut database::connection().unwrap()).unwrap().into_iter()
            .filter(|country| country.matches_filter(&filter))
            .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
            .collect()
    }
}

pub struct CountryInfo {
    country: Country,
}

#[derive(Debug)]
pub enum CountryInfoMessage {}

impl CountryInfo {

    pub fn new(country: Country) -> Self {
        Self {
            country,
        }
    }

    pub fn view(&self) -> iced::Element<'_, CountryInfoMessage> {
        column!(
            iced::widget::text(&self.country.name)
            .size(25)
            .width(iced::Length::Fixed(200.0)),
        ).into()
    }
}

#[derive(Debug)]
pub enum WorldMapCountryFilter {
    _Exclude(Vec<String>),
    Include(Vec<String>),
}

impl WorldMapCountryFilter {
    fn accept(&self, country: &Country) -> bool {
        match self {
            WorldMapCountryFilter::_Exclude(exclusions) => !exclusions.contains(&country.iso2),
            WorldMapCountryFilter::Include(inclusions) => inclusions.contains(&country.iso2),
        }
    }
}

#[derive(Debug)]
pub enum WorldMapMessage {
    FilterChanged(WorldMapCountryFilter),
    FilterRemoved,
}

pub struct WorldMap {
    country_filter: Option<WorldMapCountryFilter>,
    countries: Vec<Country>,
    countries_cache: canvas::Cache,
}

impl WorldMap {
    pub fn new() -> Self {
        Self {
            country_filter: None,
            countries: database::all_countries(&mut database::connection().unwrap()).unwrap(),
            countries_cache: canvas::Cache::default(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, WorldMapMessage> {
        canvas(self)
            .height(iced::Length::Fill)
            .width(iced::Length::Fill)
            .into()
    }

    pub fn update(&mut self, msg: WorldMapMessage) {
        self.countries_cache.clear();
        match msg {
            WorldMapMessage::FilterChanged(filter) => self.country_filter = Some(filter),
            WorldMapMessage::FilterRemoved => self.country_filter = None,
        }
    }
}

impl<Message> canvas::Program<Message> for WorldMap {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry> {
        let country_geom = self.countries_cache.draw(renderer, bounds.size(), |frame| {
            let countries_to_show = if let Some(filter) = &self.country_filter {
                self.countries.iter().filter(|c| filter.accept(c)).collect_vec()
            } else {
                self.countries.iter().collect_vec()
            };
            for country in countries_to_show {
                draw_country(country, frame);
            }
        });
        vec![country_geom]
    }
}

fn draw_country(country: &Country, frame: &mut Frame) {
    if let Some(polygons) = COUNTRY_POLYGONS.get(&country.iso2) {
        for polygon in polygons {
            let iso3_bytes = country.iso3.as_bytes();
            let color = Color::from_rgb8(iso3_bytes[0], iso3_bytes[1], iso3_bytes[2]);
            draw_polygon(polygon, color, frame);
        }
    }
}

fn draw_polygon(polygon: &Polygon, color: Color, frame: &mut Frame) {
    let frame_size = Point(frame.width(), frame.height());
    if polygon.0.is_empty() {
        return;
    }
    let points: Vec<Point> = polygon.iter()
        .map(|point| point.scale(frame_size.0, frame_size.1))
        .collect();
    let country = Path::new(|path| {
        let mut points = points.iter();
        let origin = points.next().unwrap();
        path.move_to(iced::Point::new(origin.0, origin.1));
        for point in points {
            path.line_to(iced::Point::new(point.0, point.1));
        }
        path.close();
    });
    // frame.fill(&country, Fill { style: fill::Style::Solid(Color::from_rgb(1.0, 0.0, 0.5)), ..Fill::default()});
    frame.fill(&country, Fill { style: fill::Style::Solid(color), ..Fill::default()});
    frame.stroke(&country, Stroke {
        style: stroke::Style::Solid(Color::from_rgb(0.0, 0.0, 0.0)),
        width: 1.0,
        ..Stroke::default()
    });
}