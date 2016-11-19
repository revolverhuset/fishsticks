#![feature(proc_macro)]
#[macro_use] extern crate diesel;

use diesel::Connection;
use diesel::sqlite::SqliteConnection;

fn main() {
    let connection = SqliteConnection::establish(".build.db")
        .expect(&format!("Error esablishing a database connection to .build.db"));

    diesel::migrations::run_pending_migrations(&connection).unwrap();
}
