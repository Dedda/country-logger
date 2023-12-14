-- Your SQL goes here
CREATE TABLE country_visits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    country_id INTEGER,
    FOREIGN KEY (country_id) REFERENCES countries(id)
)