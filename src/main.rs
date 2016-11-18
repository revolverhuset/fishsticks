#![feature(proc_macro)]
#![feature(custom_attribute)]
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate serde_derive;

mod ingest;
mod models;
mod schema;
mod takedown;

use diesel::Connection;
use diesel::sqlite::SqliteConnection;

fn main() {
    let database = "dev.db";

    let connection = SqliteConnection::establish(database)
        .expect(&format!("Error connecting to database at {}", database));

    let take_menu = takedown::read_menu_from_file("take.json").unwrap();
    connection.transaction(|| ingest::resturant(&connection, "Take", &take_menu)).unwrap();

    use schema::resturants::dsl::*;
    use diesel::LoadDsl;
    let results = resturants.load::<models::Resturant>(&connection).expect("Error querying db");

    for resturant in results {
        println!("{}: {}", resturant.id, &resturant.name);
    }
}
