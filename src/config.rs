extern crate getopts;

use std::env;
use std::io;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Getopts(err: getopts::Fail) { from() }
    }
}

pub struct DbConfig {
    pub connection_string: String,
    pub run_migrations: bool,
}

pub struct Config {
    pub database: DbConfig,
}

pub enum ConfigResult {
    Some(Config),
    Help,
    Err(Error)
}

const USAGE: &'static str = "Usage: fishsticks [options]";

fn create_opts() -> getopts::Options {
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.reqopt("d", "database", "specify database file. Use the special \
        value :memory: to use a volatile in-memory database", "DATABASE");
    opts.optflag("", "migrations", "run pending database migrations");
    opts
}

pub fn write_help<T: io::Write>(out: &mut T) -> io::Result<()> {
    write!(out, "{}", create_opts().usage(USAGE))
}

pub fn read_config() -> ConfigResult {
    let matches = match create_opts().parse(env::args().skip(1)) {
        Ok(matches) => matches,
        Err(err) => return ConfigResult::Err(err.into()),
    };

    if matches.opt_present("h") {
        return ConfigResult::Help;
    }

    ConfigResult::Some(Config {
        database: DbConfig {
            connection_string: matches.opt_str("database").unwrap_or(":memory:".to_owned()),
            run_migrations: matches.opt_present("migrations"),
        },
    })
}
