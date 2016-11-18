CREATE TABLE resturants (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE phone_numbers (
    resturant INTEGER NOT NULL,
    number TEXT NOT NULL,
    FOREIGN KEY(resturant) REFERENCES resturants(id),
    PRIMARY KEY(resturant, number)
);

CREATE TABLE email_addresses (
    resturant INTEGER NOT NULL,
    email_address TEXT NOT NULL,
    FOREIGN KEY(resturant) REFERENCES resturants(id),
    PRIMARY KEY(resturant, email_address)
);

CREATE TABLE menu_items (
    resturant INTEGER NOT NULL,
    id INTEGER NOT NULL,
    name TEXT NOT NULL,
    price_in_cents INTEGER NOT NULL,
    FOREIGN KEY(resturant) REFERENCES resturants(id),
    PRIMARY KEY(resturant, id)
);
