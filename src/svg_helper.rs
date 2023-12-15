use std::collections::HashMap;
use std::iter::Peekable;
use std::ops::{Add, Mul};
use std::slice::Iter;
use include_dir::{Dir, include_dir};
use lazy_static::lazy_static;
use svg::node::element::path::{Command, Data, Number, Position};
use svg::parser::Event;

static SVG_FILES: Dir = include_dir!("src/svg");
pub const SVG_WIDTH: f32 = 2000.0;
pub const SVG_HEIGT: f32 = 857.0;

lazy_static! {
        pub static ref COUNTRY_POLYGONS: HashMap<String, Vec<Polygon>> = {
            let mut map = HashMap::new();
            for file in SVG_FILES.files() {
                let name = file.path().file_name().unwrap().to_str().unwrap()[0..2].to_string();
                let contents = file.contents_utf8().unwrap().to_string();
                let polygons = polygons_from_svg(&contents);
                map.insert(name, polygons.into_iter().map(scale_polygon).collect());
            }
            map
    };
}

#[derive(Clone)]
pub struct Point(pub f32, pub f32);

impl Add<&Point> for Point {
    type Output = Point;

    fn add(self, rhs: &Point) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Mul<&Point> for Point {
    type Output = Point;

    fn mul(self, rhs: &Point) -> Self::Output {
        Point(self.0 * rhs.0, self.1 * rhs.1)
    }
}

pub struct Polygon(pub Vec<Point>);

impl Polygon {
    pub fn iter(&self) -> Iter<'_, Point> {
        self.0.iter()
    }
}

fn polygons_from_svg(source: &str) -> Vec<Polygon> {
    let svg = svg::read(source).unwrap();
    let mut paths: Vec<Polygon> = vec![];
    for event in svg {
        if let Event::Tag(svg::node::element::tag::Path, _, attributes) = event {
            let data = attributes.get("d");
            if data.is_none() {
                continue;
            }
            let data = Data::parse(data.unwrap()).unwrap();
            let mut current_path: Vec<Point> = vec![];
            let mut last_point: Option<Point> = None;
            for command in data.iter() {
                match command {
                    Command::Move(pos, params) => {
                        let mut iterator = params.iter().peekable();
                        match pos {
                            Position::Absolute => {
                                last_point = Some(point_from_iter(&mut iterator));
                                current_path.push(last_point.clone().unwrap());
                            }
                            Position::Relative => {
                                paths.push(Polygon(current_path.clone()));
                                current_path.clear();
                                last_point = Some(rel_point_from_iter(&mut iterator, &last_point.unwrap()));
                                current_path.push(last_point.clone().unwrap());
                            }
                        }
                    }
                    Command::Line(pos, params) => {
                        let mut iterator = params.iter().peekable();
                        match pos {
                            Position::Absolute => {
                                loop {
                                    last_point = Some(point_from_iter(&mut iterator));
                                    current_path.push(last_point.clone().unwrap());
                                    if iterator.peek().is_none() {
                                        break;
                                    }
                                }
                            }
                            Position::Relative => {
                                loop {
                                    last_point = Some(rel_point_from_iter(&mut iterator, &last_point.unwrap()));
                                    current_path.push(last_point.clone().unwrap());
                                    if iterator.peek().is_none() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Command::Close => {
                        paths.push(Polygon(current_path.clone()));
                        current_path.clear();
                    }
                    x => println!("Unsupported svg event: {:?}", x)
                }
            }
        }
    }
    paths
}

fn scale_polygon(polygon: Polygon) -> Polygon {
    Polygon(polygon.0.into_iter()
        .map(|point| Point(point.0 / SVG_WIDTH, point.1 / SVG_HEIGT))
        .collect())
}

fn point_from_iter(iter: &mut Peekable<Iter<Number>>) -> Point {
    Point(*iter.next().unwrap(), *iter.next().unwrap())
}

fn rel_point_from_iter(iter: &mut Peekable<Iter<Number>>, reference: &Point) -> Point {
    let abs = point_from_iter(iter);
    Point(abs.0 + reference.0, abs.1 + reference.1)
}