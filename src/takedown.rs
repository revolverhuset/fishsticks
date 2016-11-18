extern crate serde_json;

use std;
use std::io::prelude::*;
use std::fs::File;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: std::io::Error) { from() }
        Serde(err: serde_json::Error) { from() }
    }
}

#[derive(Deserialize, Debug)]
pub struct MenuItem {
    pub number: i32,
    pub name: String,
    pub price: f64
}

#[derive(Deserialize, Debug)]
pub struct Category {
    pub category: String,
    pub entries: Vec<MenuItem>
}

pub type Menu = Vec<Category>;

pub fn read_menu_from_file(filename: &str) -> Result<Menu, Error> {
    let mut f = File::open(filename)?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;
    Ok(serde_json::from_str(&buffer)?)
}
