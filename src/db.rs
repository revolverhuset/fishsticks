use diesel::Connection;
use diesel::sqlite::SqliteConnection;

embed_migrations!();

pub fn connect_database(connection_string: &str, run_migrations: bool) -> SqliteConnection {
    let connection = SqliteConnection::establish(connection_string)
        .expect(&format!("Error connecting to database at {}", connection_string));

    if run_migrations {
        embedded_migrations::run(&connection).unwrap();
    }

    connection
}

#[cfg(test)]
mod tests {
    #[test]
    fn migrations_work() {
        super::connect_database(":memory:", true);
    }
}
