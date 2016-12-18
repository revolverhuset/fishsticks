use diesel;
use diesel::sqlite::SqliteConnection;
use schema::{menus, menu_items};
use models::{Menu, MenuId};
use takedown;

use diesel::prelude::*;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Diesel(err: diesel::result::Error) { from() }
    }
}

#[derive(Insertable)]
#[table_name="menus"]
struct NewMenu {
    restaurant: i32,
}

#[derive(Insertable, Debug)]
#[table_name="menu_items"]
struct NewMenuItem<'a> {
    menu: i32,
    number: i32,
    name: &'a str,
    price_in_cents: i32
}

pub fn menu(connection: &SqliteConnection, restaurant_id: i32, menu: &takedown::Menu) -> Result<MenuId, Error> {
    use schema::menus;

    let new_menu = NewMenu { restaurant: restaurant_id };
    diesel::insert(&new_menu).into(menus::table)
        .execute(connection)?;

    let menu_id = menus::table
        .filter(menus::restaurant.eq(restaurant_id))
        .order(menus::imported.desc())
        .limit(1)
        .load::<Menu>(connection)?
        .pop().unwrap()
        .id;

    let menu_items_to_insert = menu.iter()
        .flat_map(|ref category| &category.entries)
        .map(|ref item| NewMenuItem {
            menu: i32::from(menu_id),
            number: item.number,
            name: &item.name,
            price_in_cents: (item.price * 100.0) as i32
        });

    /* Bah, Diesel does not support batch inserts for sqlite,
        see https://github.com/diesel-rs/diesel/pull/166 */

    for new_menu_item in menu_items_to_insert {
        println!("{:?}", &new_menu_item);

        diesel::insert(&new_menu_item)
            .into(menu_items::table)
            .execute(connection)?;
    }

    Ok(menu_id)
}
