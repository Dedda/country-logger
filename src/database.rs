use std::env;
use clap::Parser;
use diesel::sqlite::Sqlite;
use diesel::{QueryDsl, RunQueryDsl, SqliteConnection};
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use homedir::get_my_home;
use lazy_static::lazy_static;
use crate::Args;
use crate::base_data::COUNTRIES;
use crate::models::{Country, NewCountry};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

lazy_static! {
    static ref POOL: Pool<ConnectionManager<SqliteConnection>> = {
        dotenv().ok();
        let args = Args::parse();
        let path = determine_database_path(&args);
        let connection_manager = ConnectionManager::<SqliteConnection>::new(path);
        let pool = Pool::builder().build(connection_manager).expect("Cannot open connection pool");
        {
            migrate(&mut pool.get().expect("Cannot get connection")).unwrap();
            let country_count: i64 = crate::schema::countries::table
                .count()
                .get_result(&mut pool.get().expect("Cannot get connection"))
                .unwrap();
            if country_count == 0 {
                populate(&mut pool.get().expect("Cannot get connection")).unwrap();
            }
        }
        pool
    };
}

#[derive(Debug)]
pub enum DatabaseError {
    R2d2(r2d2::Error),
    Unknown(String),
}

impl From<r2d2::Error> for DatabaseError {
    fn from(value: r2d2::Error) -> Self {
        DatabaseError::R2d2(value)
    }
}

pub fn connection() -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, DatabaseError> {
    let connection = POOL.get()?;
    Ok(connection)
}

fn determine_database_path(args: &Args) -> String {
    let home = get_my_home().unwrap().unwrap();
    args.database_path.clone()
        .or(env::var("DATABASE_URL").ok())
        .unwrap_or(home.join(".country_logger")
            .join("da.sqlite")
            .to_str().unwrap()
            .to_string())
}

fn migrate(connection: &mut impl MigrationHarness<Sqlite>) -> Result<(), DatabaseError> {
    println!("Migrating database...");
    connection.run_pending_migrations(MIGRATIONS).map_err(|e| DatabaseError::Unknown(e.to_string()))?;
    Ok(())
}

fn populate(connection: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::countries;
    println!("Populating new database...");
    let new_countries: Vec<NewCountry> = COUNTRIES.iter()
        .map(|(_, c)| NewCountry::from(c.clone()))
        .collect();
    for new_country in new_countries.into_iter() {
        diesel::insert_into(countries::table)
            .values(&new_country)
            .execute(connection)?;
    }
    Ok(())
}

pub fn all_countries(connection: &mut SqliteConnection) -> Result<Vec<Country>, diesel::result::Error> {
    use crate::schema::countries::dsl::*;
    countries.load::<Country>(connection)
}