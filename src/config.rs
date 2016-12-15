extern crate getopts;
extern crate serde_json;

use std::env;
use std::io;
use std::fs;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Getopts(err: getopts::Fail) { from() }
        IoError(err: io::Error) { from() }
        SerdeJsonError(err: serde_json::Error) { from() }
    }
}

#[derive(Deserialize, Debug)]
pub struct DbConfig {
    #[serde(default="default_connection_string")]
    pub connection_string: String,

    #[serde(default)]
    pub run_migrations: bool
}
fn default_connection_string() -> String { ":memory:".to_owned() }

impl DbConfig {
    fn new() -> DbConfig {
        DbConfig {
            connection_string: default_connection_string(),
            run_migrations: false,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct WebConfig {
    #[serde(default="default_bind")]
    pub bind: String,

    #[serde(default="default_base")]
    pub base: String,

    pub slack_token: Option<String>,

    pub sharebill_url: Option<String>,
}
fn default_bind() -> String { "localhost:3000".to_owned() }
fn default_base() -> String { "http://localhost:3000/".to_owned() }

impl WebConfig {
    fn new() -> WebConfig {
        WebConfig {
            bind: default_bind(),
            base: default_base(),
            slack_token: None,
            sharebill_url: None,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "DbConfig::new")]
    pub database: DbConfig,

    #[serde(default = "WebConfig::new")]
    pub web: WebConfig,
}

impl Config {
    fn new() -> Config {
        Config {
            database: DbConfig {
                connection_string: default_connection_string(),
                run_migrations: false, 
            },
            web: WebConfig {
                bind: default_bind(),
                base: default_base(),
                slack_token: None,
                sharebill_url: None,
            },
        }
    }
}

pub enum ConfigResult {
    Some(Config),
    Help,
    Err(Error)
}

const USAGE: &'static str = "Usage: fishsticks [options] CONFIG_FILE...";

fn create_opts() -> getopts::Options {
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("d", "database", "specify database file. Use the special \
        value :memory: to use a volatile in-memory database", "DATABASE");
    opts.optflag("", "migrations", "run pending database migrations");
    opts.optopt("", "bind", "specify bind address for the http server. The \
        default value is localhost:3000", "ADDRESS");
    opts
}

pub fn write_help<T: io::Write>(out: &mut T) -> io::Result<()> {
    write!(out, "{}", create_opts().usage(USAGE))
}

fn read_config_file(filename: &str) -> Result<Config, Error> {
    use std::io::Read;

    let mut file = fs::File::open(filename)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    Ok(serde_json::from_str(&data)?)
}

pub fn read_config() -> ConfigResult {
    let matches = match create_opts().parse(env::args().skip(1)) {
        Ok(matches) => matches,
        Err(err) => return ConfigResult::Err(err.into()),
    };

    if matches.opt_present("h") {
        return ConfigResult::Help;
    }

    let mut cfg = Config::new();

    for ref config_file in &matches.free {
        match read_config_file(config_file) {
            Ok(config) => cfg = config,
            Err(err) => return ConfigResult::Err(err.into()),
        }
    }

    ConfigResult::Some(Config {
        database: DbConfig {
            connection_string: matches.opt_str("database").unwrap_or(cfg.database.connection_string),
            run_migrations: matches.opt_present("migrations") || cfg.database.run_migrations,
        },
        web: WebConfig {
            bind: matches.opt_str("bind").unwrap_or(cfg.web.bind),
            base: cfg.web.base,
            slack_token: cfg.web.slack_token,
            sharebill_url: cfg.web.sharebill_url,
        },
    })
}
