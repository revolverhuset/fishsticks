PRAGMA foreign_keys=OFF;

CREATE TABLE new_orders (
    id INTEGER PRIMARY KEY NOT NULL,
    restaurant INTEGER NOT NULL,
    overhead_in_cents INTEGER NOT NULL,
    opened INTEGER NOT NULL,
    closed INTEGER,
    FOREIGN KEY(restaurant) REFERENCES restaurants(id)
);

INSERT INTO new_orders SELECT * FROM orders;

DROP TABLE orders;
ALTER TABLE new_orders RENAME TO orders;

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

DROP TABLE menus;

PRAGMA foreign_key_check;

PRAGMA foreign_keys=ON;
