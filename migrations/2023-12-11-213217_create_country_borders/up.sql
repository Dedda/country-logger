-- Your SQL goes here
CREATE TABLE country_borders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    country_id INTEGER NOT NULL,
    polygon_data TEXT NOT NULL,
    FOREIGN KEY (country_id) REFERENCES countries(id)
)