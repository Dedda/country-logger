-- Your SQL goes here
CREATE TABLE country_visits (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    country_id INTEGER NOT NULL UNIQUE,
    FOREIGN KEY (country_id) REFERENCES countries(id)
)