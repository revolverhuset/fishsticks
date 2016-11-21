extern crate handlebars_iron;
extern crate iron;
extern crate router;
extern crate serde_json;

use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;
use state;

use self::handlebars_iron::{Template, HandlebarsEngine, DirectorySource};
use self::iron::prelude::*;
use self::iron::{status, typemap, BeforeMiddleware};
use self::router::Router;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Bummer
        Handlebars(err: handlebars_iron::SourceError) { from() }
    }
}

struct StateContainer(pub Arc<Mutex<state::State>>);

impl typemap::Key for StateContainer { type Value = StateContainer; }

impl BeforeMiddleware for StateContainer {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<StateContainer>(StateContainer(self.0.clone()));
        Ok(())
    }
}

fn index(req: &mut Request) -> IronResult<Response> {
    use self::serde_json::value::{self, Value};
    let state = req.extensions.get::<StateContainer>().unwrap().0.lock().unwrap();

    let mut data = BTreeMap::<String, Value>::new();

    // TODO Understand error handling with Iron and use ? here:
    let resturants = state.resturants().unwrap();

    data.insert("resturants".to_string(), value::to_value(&resturants));

    Ok(Response::with((status::Ok, Template::new("index", data))))
}

fn menu(req: &mut Request) -> IronResult<Response> {
    use self::serde_json::value::{self, Value};
    let state = req.extensions.get::<StateContainer>().unwrap().0.lock().unwrap();

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
    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./templates/", ".hbs")));
    hbse.reload()?;

    let mut router = Router::new();
    router.get("/", index, "index");
    router.get("/resturant/:id", menu, "menu");

    let mut chain = Chain::new(router);
    chain.link_before(StateContainer(Arc::new(Mutex::new(state))));
    chain.link_after(hbse);

    let listening = Iron::new(chain).http(bind).map_err(|_| Error::Bummer)?;
    println!("Listening to {:?}", &listening.socket);
    drop(listening); // Will implicitly block and keep handling requests

    Ok(())
}
