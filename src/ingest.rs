use diesel;
use diesel::sqlite::SqliteConnection;
use schema::{restaurants, menu_items};
use models::Restaurant;
use takedown;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Diesel(err: diesel::result::Error) { from() }
    }
}

#[derive(Insertable)]
#[table_name="restaurants"]
struct NewRestaurant<'a> {
    name: &'a str
}

#[derive(Insertable)]
#[table_name="menu_items"]
struct NewMenuItem<'a> {
    restaurant: i32,
    id: i32,
    name: &'a str,
    price_in_cents: i32
}

pub fn restaurant(connection: &SqliteConnection, name: &str, menu: &takedown::Menu) -> Result<(), Error> {
    use schema::restaurants;
    use diesel::prelude::*;
    let new_restaurant = NewRestaurant {
        name: name
    };
    diesel::insert(&new_restaurant).into(restaurants::table)
        .execute(connection)?;

    let restaurant_id = restaurants::dsl::restaurants
        .filter(restaurants::dsl::name.eq(name))
        .load::<Restaurant>(connection)?
        [0].id;

    let menu_items_to_insert = menu.iter()
        .flat_map(|ref category| &category.entries)
        .map(|ref item| NewMenuItem {
            restaurant: restaurant_id,
            id: item.number,
            name: &item.name,
            price_in_cents: (item.price * 100.0) as i32
        });

    /* Bah, Diesel does not support batch inserts for sqlite,
        see https://github.com/diesel-rs/diesel/pull/166 */

    for new_menu_item in menu_items_to_insert {
        diesel::insert(&new_menu_item)
            .into(menu_items::table)
            .execute(connection)?;
    }

    Ok(())
}
