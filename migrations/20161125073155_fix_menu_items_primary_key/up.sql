PRAGMA foreign_keys=OFF;

CREATE TABLE new_menu_items (
    id INTEGER PRIMARY KEY NOT NULL,
    restaurant INTEGER NOT NULL,
    'number' INTEGER NOT NULL,
    name TEXT NOT NULL COLLATE NOCASE,
    price_in_cents INTEGER NOT NULL,
    FOREIGN KEY(restaurant) REFERENCES restaurants(id),
    UNIQUE (restaurant, 'number')
);

INSERT INTO new_menu_items
    (restaurant, "number", name, price_in_cents)
SELECT
    restaurant, id, name, price_in_cents
FROM menu_items;

DROP TABLE menu_items;
ALTER TABLE new_menu_items RENAME TO menu_items;

CREATE TABLE new_order_items (
    id INTEGER PRIMARY KEY NOT NULL,
    'order' INTEGER NOT NULL,
    person_name TEXT NOT NULL,
    menu_item INTEGER NOT NULL,
    FOREIGN KEY('order') REFERENCES orders(id),
    FOREIGN KEY(menu_item) REFERENCES menu_items(id)

    -- Missing constraint: It should only be possible to add orders from the
    -- correct menu, CHECK order.restaurant = menu_item.restaurant
);

INSERT INTO new_order_items SELECT
    order_items.id, order_items."order", order_items.person_name, menu_items.id
FROM
    order_items
        JOIN orders ON order_items."order" = orders.id
        JOIN menu_items ON
            orders.restaurant = menu_items.restaurant
            AND menu_items.number = order_items.menu_item;

DROP TABLE order_items;
ALTER TABLE new_order_items RENAME TO order_items;

PRAGMA foreign_key_check;

PRAGMA foreign_keys=ON;
