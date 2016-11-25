PRAGMA foreign_keys=OFF;

CREATE TABLE new_order_items (
    id INTEGER PRIMARY KEY NOT NULL,
    'order' INTEGER NOT NULL,
    person_name TEXT NOT NULL,
    menu_item INTEGER NOT NULL,
    FOREIGN KEY('order') REFERENCES orders(id),
    FOREIGN KEY(menu_item) REFERENCES menu_items(id)
);

INSERT INTO new_order_items SELECT
    order_items.id, order_items."order", order_items.person_name, menu_items.number
FROM
    order_items JOIN menu_items ON order_items.menu_item = menu_items.id;

DROP TABLE order_items;
ALTER TABLE new_order_items RENAME TO order_items;


CREATE TABLE new_menu_items (
    restaurant INTEGER NOT NULL,
    id INTEGER NOT NULL,
    name TEXT NOT NULL,
    price_in_cents INTEGER NOT NULL,
    FOREIGN KEY(restaurant) REFERENCES restaurants(id),
    PRIMARY KEY(restaurant, id)
);

INSERT INTO new_menu_items
    (restaurant, id, name, price_in_cents)
SELECT
    restaurant, "number", name, price_in_cents
FROM menu_items;

DROP TABLE menu_items;
ALTER TABLE new_menu_items RENAME TO menu_items;

-- Leave broken foreign_keys OFF
