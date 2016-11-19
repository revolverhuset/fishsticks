extern crate getopts;

use std::env;
use std::io;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
    }
}

pub struct Config {
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
    opts
}

pub fn write_help<T: io::Write>(out: &mut T) {
    write!(out, "{}", create_opts().usage(USAGE));
}

pub fn read_config() -> ConfigResult {
    let args = env::args().collect::<Vec<_>>();

    let mut opts = create_opts();

    let matches = opts.parse(&args[1..]).unwrap();
    if matches.opt_present("h") {
        return ConfigResult::Help;
    }

    ConfigResult::Some(Config{})
}
