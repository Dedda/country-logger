-- Your SQL goes here
ALTER TABLE countries ADD description TEXT;

CREATE TABLE country_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    country_id INTEGER NOT NULL,
    note TEXT,
    done BOOLEAN,
    FOREIGN KEY (country_id) REFERENCES countries(id)
);