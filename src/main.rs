#[macro_use]
extern crate bart_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;
extern crate crossbeam;
extern crate iron;
extern crate itertools;
extern crate matrix_bot_api;
extern crate num;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate sharebill;
extern crate time;
extern crate urlencoded;
extern crate uuid;

mod cmd;
mod config;
mod db;
mod ingest;
mod matrix;
mod models;
mod schema;
mod slack;
mod state;
mod takedown;
mod web;
mod words;

use std::sync::{Arc, Mutex};

fn main() {
    let config = match config::read_config() {
        config::ConfigResult::Some(config) => config,
        config::ConfigResult::Help => {
            config::write_help(&mut std::io::stdout()).unwrap();
            return;
        }
        config::ConfigResult::Err(err) => {
            println!("{:?}", &err);
            panic!(err)
        }
    };

    let db_connection = db::connect_database(
        &config.database.connection_string,
        config.database.run_migrations,
    );
    let state = Arc::new(Mutex::new(state::State::new(db_connection)));

    crossbeam::scope(|scope| {
        let web = {
            let state = state.clone();
            let config = config.clone();
            scope.spawn(|| {
                web::run(
                    state,
                    &config.web.bind,
                    config.web.base,
                    config.web.slack_token,
                    config.web.sharebill_url,
                    config.web.sharebill_cookies,
                )
            })
        };

        let env = web::Env {
            base_url: config.web.base,
            maybe_sharebill_url: config.web.sharebill_url,
            sharebill_cookies: config.web.sharebill_cookies,
        };

        let matrix = config.matrix.map(|matrix| {
            scope.spawn(move || {
                matrix::run(state, env, &matrix.user, &matrix.password, &matrix.server)
            })
        });

        web.join().unwrap();
        matrix.map(|x| x.join().unwrap());
    });
}
