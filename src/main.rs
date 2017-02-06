#[macro_use] extern crate bart_derive;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate serde_derive;
extern crate sharebill;

mod config;
mod db;
mod ingest;
mod models;
mod schema;
mod slack;
mod state;
mod takedown;
mod web;
mod words;

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

    let db_connection = db::connect_database(&config.database.connection_string, config.database.run_migrations);
    let state = state::State::new(db_connection);

    web::run(
        state,
        &config.web.bind,
        config.web.base,
        config.web.slack_token,
        config.web.sharebill_url,
    ).unwrap();
}
