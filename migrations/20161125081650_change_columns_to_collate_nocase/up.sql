PRAGMA foreign_keys=OFF;

CREATE TABLE new_restaurants (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT UNIQUE NOT NULL COLLATE NOCASE
);

INSERT INTO new_restaurants SELECT * FROM restaurants;

DROP TABLE restaurants;
ALTER TABLE new_restaurants RENAME TO restaurants;

CREATE TABLE new_menu_items (
    id INTEGER PRIMARY KEY NOT NULL,
    restaurant INTEGER NOT NULL,
    'number' INTEGER NOT NULL,
    name TEXT NOT NULL COLLATE NOCASE,
    price_in_cents INTEGER NOT NULL,
    FOREIGN KEY(restaurant) REFERENCES restaurants(id),
    UNIQUE (restaurant, 'number')
);

INSERT INTO new_menu_items SELECT * FROM menu_items;

DROP TABLE menu_items;
ALTER TABLE new_menu_items RENAME TO menu_items;

PRAGMA foreign_key_check;

PRAGMA foreign_keys=ON;
