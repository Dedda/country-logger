use iced::widget::{checkbox, column, row};
use itertools::Itertools;
use crate::database;
use crate::models::Country;

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

pub mod world_map {
    use iced::{Color, Rectangle};
    use iced::mouse::Cursor;
    use iced::widget::canvas;
    use iced::widget::canvas::{Fill, fill, Frame, Path, Stroke, stroke};
    use crate::database;
    use crate::models::Country;
    use crate::svg_helper::{COUNTRY_POLYGONS, Point, Polygon, SVG_HEIGT, SVG_WIDTH};

    #[derive(Debug)]
    pub enum WorldMapCountryFilter {
        Include(Vec<String>),
    }

    impl WorldMapCountryFilter {
        fn accept(&self, country: &Country) -> bool {
            match self {
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
        countries: Vec<(Country, bool)>,
        countries_cache: canvas::Cache,
    }

    impl WorldMap {
        pub fn new() -> Self {
            Self {
                country_filter: None,
                countries: database::all_countries_with_visit_status(&mut database::require_connection()).unwrap(),
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
            self.countries = database::all_countries_with_visit_status(&mut database::require_connection()).unwrap();
            self.countries_cache.clear();
            match msg {
                WorldMapMessage::FilterChanged(filter) => self.country_filter = Some(filter),
                WorldMapMessage::FilterRemoved => self.country_filter = None,
            }
        }
    }

    enum CountryRenderStyle {
        Normal(bool),
        Selected,
        Unselected,
    }

    impl CountryRenderStyle {
        fn get_fill_color(&self, country: &Country) -> Color {
            match self {
                CountryRenderStyle::Normal(visited) => {
                    let iso3_bytes = country.iso3.as_bytes();
                    if *visited {
                        color_from_ascii(iso3_bytes)
                    } else {
                        let iso3_bytes: Vec<u8> = iso3_bytes.iter()
                            .map(|c| {
                                let diff = b'Z' - c;
                                b'Z' - (diff / 8)
                            })
                            .collect();
                        color_from_ascii(&iso3_bytes)
                    }
                }
                CountryRenderStyle::Selected => {
                    let iso3_bytes = country.iso3.as_bytes();
                    color_from_ascii(iso3_bytes)
                }
                CountryRenderStyle::Unselected => {
                    Color::from_rgb(0.7, 0.7, 0.7)
                }
            }
        }

        fn get_stroke_style(&self) -> stroke::Style {
            match self {
                CountryRenderStyle::Normal(_) => {
                    stroke::Style::Solid(Color::from_rgb(0.0, 0.0, 0.0))
                }
                CountryRenderStyle::Selected => {
                    stroke::Style::Solid(Color::from_rgb(0.0, 1.0, 0.0))
                }
                CountryRenderStyle::Unselected => {
                    stroke::Style::Solid(Color::from_rgb(0.0, 0.0, 0.0))
                }
            }
        }
    }

    fn color_from_ascii(ascii: &[u8]) -> Color {
        Color::from_rgb(
            color_component_from_ascii(&ascii[0]),
            color_component_from_ascii(&ascii[1]),
            color_component_from_ascii(&ascii[2]),
        )
    }

    fn color_component_from_ascii(ascii: &u8) -> f32 {
        let ascii = ascii - b'A';
        let ascii = ascii as f32;
        ascii / 26.0
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
                if let Some(filter) = &self.country_filter {
                    let (selected, unselected): (Vec<&Country>, Vec<&Country>) = self.countries.iter()
                        .map(|(country, _visited)| country)
                        .partition(|country| filter.accept(country));
                    for country in unselected {
                        draw_country(country, CountryRenderStyle::Unselected, frame);
                    }
                    for country in selected {
                        draw_country(country, CountryRenderStyle::Selected, frame);
                    }
                } else {
                    for (country, visited) in &self.countries {
                        draw_country(country, CountryRenderStyle::Normal(*visited), frame);
                    }
                }
            });
            vec![country_geom]
        }
    }

    fn draw_country(country: &Country, style: CountryRenderStyle, frame: &mut Frame) {
        if let Some(polygons) = COUNTRY_POLYGONS.get(&country.iso2) {
            for polygon in polygons {
                draw_polygon(polygon, country, &style, frame);
            }
        }
    }

    fn draw_polygon(polygon: &Polygon, country: &Country, style: &CountryRenderStyle, frame: &mut Frame) {
        let aspect_ratio_svg = SVG_WIDTH / SVG_HEIGT;
        let aspect_ratio_frame = frame.width() / frame.height();
        let map_render_size: Point;
        let map_render_offset: Point;
        if aspect_ratio_svg > aspect_ratio_frame {
            map_render_size = Point(frame.width(), frame.width() / aspect_ratio_svg);
            map_render_offset = Point(0.0, (frame.height() - map_render_size.1) / 2.0);
        } else {
            map_render_size = Point(frame.height() * aspect_ratio_svg, frame.height());
            map_render_offset = Point((frame.width() - map_render_size.0) / 2.0, 0.0);
        }
        if polygon.0.is_empty() {
            return;
        }
        let points: Vec<Point> = polygon.iter()
            .map(|point| point.clone() * &map_render_size)
            .map(|point| point + &map_render_offset)
            .collect();
        let path = Path::new(|path| {
            let mut points = points.iter();
            let origin = points.next().unwrap();
            path.move_to(iced::Point::new(origin.0, origin.1));
            for point in points {
                path.line_to(iced::Point::new(point.0, point.1));
            }
            path.close();
        });
        let color = style.get_fill_color(country);
        frame.fill(&path, Fill { style: fill::Style::Solid(color), ..Fill::default() });
        frame.stroke(&path, Stroke {
            style: style.get_stroke_style(),
            width: 1.0,
            ..Stroke::default()
        });
    }
}