CREATE TABLE orders (
    id INTEGER PRIMARY KEY NOT NULL,
    restaurant INTEGER NOT NULL,
    overhead_in_cents INTEGER NOT NULL,

    -- opened and closed are timestamps in Unix time. They should
    -- have been of type DATETIME, but Diesel does not support that
    opened INTEGER NOT NULL,
    closed INTEGER,

    FOREIGN KEY(restaurant) REFERENCES restaurants(id)
);

CREATE TABLE order_items (
    id INTEGER PRIMARY KEY NOT NULL,
    'order' INTEGER NOT NULL,
    person_name TEXT NOT NULL,
    menu_item INTEGER NOT NULL,
    FOREIGN KEY('order') REFERENCES orders(id),
    FOREIGN KEY(menu_item) REFERENCES menu_items(id)
);
