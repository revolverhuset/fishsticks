CREATE TABLE restaurants (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE phone_numbers (
    restaurant INTEGER NOT NULL,
    number TEXT NOT NULL,
    FOREIGN KEY(restaurant) REFERENCES restaurants(id),
    PRIMARY KEY(restaurant, number)
);

CREATE TABLE email_addresses (
    restaurant INTEGER NOT NULL,
    email_address TEXT NOT NULL,
    FOREIGN KEY(restaurant) REFERENCES restaurants(id),
    PRIMARY KEY(restaurant, email_address)
);

CREATE TABLE menu_items (
    restaurant INTEGER NOT NULL,
    id INTEGER NOT NULL,
    name TEXT NOT NULL,
    price_in_cents INTEGER NOT NULL,
    FOREIGN KEY(restaurant) REFERENCES restaurants(id),
    PRIMARY KEY(restaurant, id)
);
