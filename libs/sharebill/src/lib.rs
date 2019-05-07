#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod models;
pub mod rational;

pub use rational::Rational;
