#[macro_use] extern crate bart_derive;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate serde_derive;
extern crate sharebill;

mod config;
mod ingest;
mod models;
mod schema;
mod slack;
mod state;
mod takedown;
mod web;
mod words;

use diesel::Connection;
use diesel::sqlite::SqliteConnection;

embed_migrations!();

fn connect_database(connection_string: &str, run_migrations: bool) -> SqliteConnection {
    let connection = SqliteConnection::establish(connection_string)
        .expect(&format!("Error connecting to database at {}", connection_string));

    if run_migrations {
        embedded_migrations::run(&connection).unwrap();
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
    let state = state::State::new(connection);

    web::run(state, &config.web.bind, config.web.base, config.web.slack_token, config.web.sharebill_url).unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn migrations_work() {
        ::connect_database(":memory:", true);
    }
}
