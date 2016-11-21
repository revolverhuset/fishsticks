extern crate serde_json;

use std;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: std::io::Error) { from() }
        Serde(err: serde_json::Error) { from() }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct MenuItem {
    pub number: i32,
    pub name: String,
    pub price: f64
}

#[derive(Deserialize, Debug, Clone)]
pub struct Category {
    pub category: String,
    pub entries: Vec<MenuItem>
}

pub type Menu = Vec<Category>;
