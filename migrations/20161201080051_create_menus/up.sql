PRAGMA foreign_keys=OFF;

CREATE TABLE menus (
    id INTEGER PRIMARY KEY NOT NULL,
    restaurant INTEGER NOT NULL,

    -- imported is Unix time. It should have been
    -- DATETIME, but Diesel does not support that
    imported INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')),

    FOREIGN KEY(restaurant) REFERENCES restaurants(id)
);

INSERT INTO menus (id, restaurant)
    SELECT id, id AS restaurant FROM restaurants;

CREATE TABLE new_menu_items (
    id INTEGER PRIMARY KEY NOT NULL,
    menu INTEGER NOT NULL,
    'number' INTEGER NOT NULL,
    name TEXT NOT NULL COLLATE NOCASE,
    price_in_cents INTEGER NOT NULL,
    FOREIGN KEY(menu) REFERENCES menus(id),
    UNIQUE (menu, 'number')
);

INSERT INTO new_menu_items SELECT * FROM menu_items;

DROP TABLE menu_items;
ALTER TABLE new_menu_items RENAME TO menu_items;

CREATE TABLE new_orders (
    id INTEGER PRIMARY KEY NOT NULL,
    menu INTEGER NOT NULL,
    overhead_in_cents INTEGER NOT NULL,
    opened INTEGER NOT NULL,
    closed INTEGER,
    FOREIGN KEY(menu) REFERENCES menus(id)
);

INSERT INTO new_orders SELECT * FROM orders;

DROP TABLE orders;
ALTER TABLE new_orders RENAME TO orders;

PRAGMA foreign_key_check;

PRAGMA foreign_keys=ON;
