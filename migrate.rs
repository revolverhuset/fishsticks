extern crate diesel;

use diesel::Connection;

fn main() {
    let _ = std::fs::remove_file(".build.db");

    let connection = diesel::sqlite::SqliteConnection::establish(".build.db")
        .expect(&format!("Error esablishing a database connection to .build.db"));

    diesel::migrations::run_pending_migrations(&connection).unwrap();
}
