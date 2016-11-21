extern crate handlebars_iron;
extern crate iron;
extern crate router;
extern crate serde_json;

use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;
use state;

use self::handlebars_iron::{Template, HandlebarsEngine, DirectorySource};
use self::iron::prelude::*;
use self::iron::status;
use self::router::Router;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Bummer
        Handlebars(err: handlebars_iron::SourceError) { from() }
    }
}

fn index(state: &mut state::State, _: &mut Request) -> IronResult<Response> {
    use self::serde_json::value::{self, Value};

    let mut data = BTreeMap::<String, Value>::new();

    // TODO Understand error handling with Iron and use ? here:
    let resturants = state.resturants().unwrap();

    data.insert("resturants".to_string(), value::to_value(&resturants));

    Ok(Response::with((status::Ok, Template::new("index", data))))
}

fn menu(state: &mut state::State, req: &mut Request) -> IronResult<Response> {
    use self::serde_json::value::{self, Value};

    let id = req.extensions.get::<Router>().unwrap()
        .find("id").unwrap()
        .parse::<i32>().unwrap();

    let mut data = BTreeMap::<String, Value>::new();

    // TODO Understand error handling with Iron and use ? here:
    let menu = state.menu(id).unwrap();

    data.insert("menu".to_string(), value::to_value(&menu));

    Ok(Response::with((status::Ok, Template::new("menu", data))))
}

pub fn run(state: state::State, bind: &str) -> Result<(), Error> {
    let shared_state = Arc::new(Mutex::new(state));
    let s1 = shared_state.clone();
    let s2 = shared_state.clone();

    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./templates/", ".hbs")));
    hbse.reload()?;

    let mut router = Router::new();
    router.get("/", move |req: &mut Request| index(&mut s1.lock().unwrap(), req), "index");
    router.get("/resturant/:id", move |req: &mut Request| menu(&mut s2.lock().unwrap(), req), "menu");

    let mut chain = Chain::new(router);
    chain.link_after(hbse);

    let listening = Iron::new(chain).http(bind).map_err(|_| Error::Bummer)?;
    println!("Listening to {:?}", &listening.socket);
    drop(listening); // Will implicitly block and keep handling requests

    Ok(())
}
