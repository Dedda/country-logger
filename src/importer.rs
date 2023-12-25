use std::fs;
use std::path::Path;
use crate::database::{country_by_iso2, DatabaseError, is_country_visited, require_connection, visit_country};

#[derive(Debug)]
pub enum ImportError {
    Database(DatabaseError),
    Io(std::io::Error),
}

impl From<DatabaseError> for ImportError {
    fn from(value: DatabaseError) -> Self {
        ImportError::Database(value)
    }
}

impl From<std::io::Error> for ImportError {
    fn from(value: std::io::Error) -> Self {
        ImportError::Io(value)
    }
}

impl From<diesel::result::Error> for ImportError {
    fn from(value: diesel::result::Error) -> Self {
        ImportError::Database(DatabaseError::Diesel(value))
    }
}

pub fn simple_import(path: &Path) -> Result<(), ImportError>{
    let string = fs::read_to_string(path)?;
    let iso2 = string.split(',');
    // let mut connection = require_connection();
    let mut countries = vec![];
    for iso2 in iso2 {
        if let Some(country) = country_by_iso2(&mut require_connection(), iso2)? {
            countries.push(country);
        }
    }
    for country in countries {
        println!("Found country! {}", country.id);
        println!("Visited? {}", is_country_visited(&mut require_connection(), &country)?);
        if !is_country_visited(&mut require_connection(), &country)? {
            println!("{} is not visited", country.iso2);
            visit_country(&mut require_connection(), &country)?;
        }
        }
    Ok(())
}