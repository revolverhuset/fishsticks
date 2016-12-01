use diesel;
use diesel::sqlite::SqliteConnection;
use schema::{restaurants, menus, menu_items};
use models::{Menu, Restaurant};
use takedown;

use diesel::prelude::*;

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
#[table_name="menus"]
struct NewMenu {
    restaurant: i32,
}

#[derive(Insertable)]
#[table_name="menu_items"]
struct NewMenuItem<'a> {
    menu: i32,
    id: i32,
    name: &'a str,
    price_in_cents: i32
}

pub fn restaurant(connection: &SqliteConnection, name: &str) -> Result<i32, Error> {
    use schema::restaurants;
    let new_restaurant = NewRestaurant {
        name: name
    };
    diesel::insert(&new_restaurant).into(restaurants::table)
        .execute(connection)?;

    let restaurant_id = restaurants::dsl::restaurants
        .filter(restaurants::dsl::name.eq(name))
        .load::<Restaurant>(connection)?
        [0].id;

    Ok(restaurant_id)
}

pub fn menu(connection: &SqliteConnection, restaurant_id: i32, menu: &takedown::Menu) -> Result<i32, Error> {
    use schema::menus;

    let new_menu = NewMenu { restaurant: restaurant_id };
    diesel::insert(&new_menu).into(menus::table)
        .execute(connection)?;

    let menu_id = menus::table
        .filter(menus::dsl::restaurant.eq(restaurant_id))
        .order(menus::dsl::imported.desc())
        .limit(1)
        .load::<Menu>(connection)?
        .pop().unwrap()
        .id;

    let menu_items_to_insert = menu.iter()
        .flat_map(|ref category| &category.entries)
        .map(|ref item| NewMenuItem {
            menu: menu_id,
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

    Ok(menu_id)
}
