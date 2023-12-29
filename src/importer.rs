use std::collections::HashMap;
use std::fs;
use std::path::Path;
use diesel::SqliteConnection;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::database;
use crate::database::{country_by_iso2, DatabaseError, is_country_visited, require_connection, visit_country};
use crate::models::{Country, CountryVisit};

const IMPORT_FILE_VERSION: i32 = 1;

#[derive(Debug)]
pub enum ImportError {
    Database(DatabaseError),
    Io(std::io::Error),
    Format(FormatError),
}

#[derive(Debug)]
pub enum FormatError {
    MissingMetaLine,
    MetaLineFormat(String),
    WrongVersion {
        expected: i32,
        actual: i32,
    },
    SerdeJson(serde_json::Error),
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

impl From<serde_json::Error> for ImportError {
    fn from(value: serde_json::Error) -> Self {
        ImportError::Format(FormatError::SerdeJson(value))
    }
}

pub fn simple_import(path: &Path) -> Result<(), ImportError> {
    let string = fs::read_to_string(path)?;
    let iso2 = string.split(',');
    let mut connection = require_connection();
    let mut countries = vec![];
    for iso2 in iso2 {
        if let Some(country) = country_by_iso2(&mut connection, iso2)? {
            countries.push(country);
        }
    }
    for country in countries {
        if !is_country_visited(&mut connection, &country)? {
            visit_country(&mut connection, &country)?;
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ImportCountry {
    id: i32,
    name: String,
    iso2: String,
    iso3: String,
    description: Option<String>,
}

impl From<ImportCountry> for Country {
    fn from(value: ImportCountry) -> Self {
        Country {
            id: value.id,
            name: value.name,
            iso2: value.iso2,
            iso3: value.iso3,
            description: value.description,
        }
    }
}

impl From<Country> for ImportCountry {
    fn from(value: Country) -> Self {
        Self {
            id: value.id,
            name: value.name,
            iso2: value.iso2,
            iso3: value.iso3,
            description: value.description,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ImportVisit {
    pub id: i32,
    pub country_id: i32,
}

impl From<ImportVisit> for CountryVisit {
    fn from(value: ImportVisit) -> Self {
        CountryVisit {
            id: value.id,
            country_id: value.country_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ImportNotes {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FullImportFile {
    countries: Vec<ImportCountry>,
    visits: Vec<ImportVisit>,
    notes: HashMap<i32, Vec<ImportNotes>>,
}

pub fn full_export(path: &Path) -> Result<(), ImportError> {
    let mut connection = require_connection();
    let full_import_file = FullImportFile {
        countries: database::all_countries(&mut connection)?.into_iter().map(Into::into).collect(),
        visits: vec![],
        notes: Default::default(),
    };
    let export_contents = format_full_export_file(full_import_file)?;
    fs::write(path, export_contents)?;
    Ok(())
}

fn format_full_export_file(full_export_file: FullImportFile) -> Result<String, ImportError> {
    let json = serde_json::to_string_pretty(&full_export_file)?;
    Ok(format!("{}\n{}", format_meta_line(), json))
}

fn format_meta_line() -> String {
    format!("{};", IMPORT_FILE_VERSION)
}

pub fn full_import(path: &Path) -> Result<(), ImportError> {
    let full_import_file = read_full_import_file(path)?;
    let FullImportFile {
        countries,
        visits,
        notes: _notes,
    } = full_import_file;
    let mut connection = require_connection();
    for country in countries {
        import_country_with_id(&mut connection, country)?;
    }
    for visit in visits {
        import_country_visit(&mut connection, visit)?;
    }
    // TODO: Add import for notes
    Ok(())
}

fn read_full_import_file(path: &Path) -> Result<FullImportFile, ImportError> {
    let file_contents = fs::read_to_string(path)?;
    full_import_file_from_string(&file_contents)
}

fn full_import_file_from_string(file_contents: &str) -> Result<FullImportFile, ImportError> {
    let mut file_contents = file_contents.lines();
    let meta_line = file_contents.next();
    if meta_line.is_none() {
        return Err(ImportError::Format(FormatError::MissingMetaLine));
    }
    let mut meta_line = meta_line.unwrap().split(';');
    if let Some(version) = meta_line.next() {
        if let Ok(version) = version.parse::<i32>() {
            if version != IMPORT_FILE_VERSION {
                return Err(ImportError::Format(FormatError::WrongVersion {
                    expected: IMPORT_FILE_VERSION,
                    actual: version,
                }))
            }
        } else {
            return Err(ImportError::Format(FormatError::MetaLineFormat("Version is not a number (i32)".to_string())));
        }
    } else {
        return Err(ImportError::Format(FormatError::MetaLineFormat("Missing version".to_string())));
    }
    let full_import_file: FullImportFile = serde_json::from_str(&file_contents.join("\n"))?;
    Ok(full_import_file)
}

fn import_country_with_id(connection: &mut SqliteConnection, country: ImportCountry) -> Result<(), DatabaseError> {
    database::import_country_with_id(connection, country.into())?;
    Ok(())
}

fn import_country_visit(connection: &mut SqliteConnection, visit: ImportVisit) -> Result<(), DatabaseError> {
    database::import_country_visit(connection, visit.into())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::importer::{format_full_export_file, full_import_file_from_string, FullImportFile, ImportCountry, ImportVisit};

    #[test]
    fn import_exported_file_gives_same_result() {
        let import_file = FullImportFile {
            countries: vec![
                ImportCountry {
                    id: 1,
                    name: "Country 1".to_string(),
                    iso2: "C1".to_string(),
                    iso3: "CY1".to_string(),
                    description: None,
                },
                ImportCountry {
                    id: 2,
                    name: "Country 2".to_string(),
                    iso2: "C2".to_string(),
                    iso3: "CY2".to_string(),
                    description: Some("Second Country".to_string()),
                },
            ],
            visits: vec![
                ImportVisit {
                    id: 1,
                    country_id: 2,
                },
                ImportVisit {
                    id: 2,
                    country_id: 1,
                },
            ],
            notes: Default::default(),
        };
        let exported = format_full_export_file(import_file.clone()).unwrap();
        let imported = full_import_file_from_string(&exported).unwrap();
        assert_eq!(import_file, imported);
    }
}