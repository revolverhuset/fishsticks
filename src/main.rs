#![feature(proc_macro)]
#![feature(custom_attribute)]
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate serde_derive;

mod config;
mod ingest;
mod models;
mod schema;
mod takedown;

use diesel::Connection;
use diesel::sqlite::SqliteConnection;

fn connect_database(connection_string: &str, run_migrations: bool) -> SqliteConnection {
    let connection = SqliteConnection::establish(connection_string)
        .expect(&format!("Error connecting to database at {}", connection_string));

    if run_migrations {
        diesel::migrations::run_pending_migrations(&connection).unwrap();
    }

    connection
}

fn main() {
    let config = match config::read_config() {
        config::ConfigResult::Some(config) => config,
        config::ConfigResult::Help => {
            config::write_help(&mut std::io::stdout()).unwrap();
            return;
        },
        config::ConfigResult::Err(err) => {
            println!("{:?}", &err);
            panic!(err)
        },
    };

    let connection = connect_database(&config.database.connection_string, config.database.run_migrations);

    let take_menu = takedown::read_menu_from_file("take.json").unwrap();
    connection.transaction(|| ingest::resturant(&connection, "Take", &take_menu)).unwrap();

    use schema::resturants::dsl::*;
    use diesel::LoadDsl;
    let results = resturants.load::<models::Resturant>(&connection).expect("Error querying db");

    for resturant in results {
        println!("{}: {}", resturant.id, &resturant.name);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn migrations_work() {
        ::connect_database(":memory:", true);
    }
}
