extern crate bodyparser;
extern crate handlebars_iron;
extern crate iron;
extern crate router;
extern crate serde_json;
extern crate urlencoded;

use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;
use slack;
use state;
use takedown;

use self::handlebars_iron::{Template, HandlebarsEngine, DirectorySource};
use self::iron::prelude::*;
use self::iron::{status, typemap, BeforeMiddleware};
use self::router::Router;
use self::urlencoded::UrlEncodedBody;

// TODO Understand error handling with Iron
quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Bummer
        Handlebars(err: handlebars_iron::SourceError) { from() }
    }
}

#[derive(Clone)]
pub struct StateContainer(pub Arc<Mutex<state::State>>);

impl typemap::Key for StateContainer { type Value = StateContainer; }

impl BeforeMiddleware for StateContainer {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<StateContainer>(self.clone());
        Ok(())
    }
}

pub struct Env {
    pub base_url: String,
}

#[derive(Clone)]
pub struct EnvContainer(pub Arc<Env>);

impl typemap::Key for EnvContainer { type Value = EnvContainer; }

impl BeforeMiddleware for EnvContainer {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<EnvContainer>(self.clone());
        Ok(())
    }
}

fn index(req: &mut Request) -> IronResult<Response> {
    use self::serde_json::value::{self, Value};
    let state = req.extensions.get::<StateContainer>().unwrap().0.lock().unwrap();

    let mut data = BTreeMap::<String, Value>::new();

    let restaurants = state.restaurants().unwrap();

    data.insert("restaurants".to_string(), value::to_value(&restaurants));

    Ok(Response::with((status::Ok, Template::new("index", data))))
}

fn create_restaurant(req: &mut Request) -> IronResult<Response> {
    use self::iron::headers::Location;
    use self::iron::modifiers::Header;
    let hashmap = req.get::<UrlEncodedBody>().unwrap();

    let name: Option<&str> =
        hashmap.get("name")
            .and_then(|x| x.get(0))
            .map(String::as_ref);

    if name.is_none() {
        return Ok(Response::with((status::BadRequest)));
    }
    let name = name.unwrap();

    let state = req.extensions.get::<StateContainer>().unwrap().0.lock().unwrap();
    let ref env = req.extensions.get::<EnvContainer>().unwrap().0;
    let id = match state.create_restaurant(name) {
        Ok(id) => id,
        Err(_) => return Ok(Response::with((
            status::InternalServerError,
            "Error!",
        )))
    };

    let created_url = format!("{}restaurant/{}", &env.base_url, id);

    Ok(Response::with((
        status::Created,
        format!("New restaurant at {}", &created_url),
        Header(Location(created_url)),
    )))
}

fn restaurant(req: &mut Request) -> IronResult<Response> {
    use self::serde_json::value::{self, Value};
    let state = req.extensions.get::<StateContainer>().unwrap().0.lock().unwrap();

    let restaurant_id = req.extensions.get::<Router>().unwrap()
        .find("id").unwrap()
        .parse::<i32>().unwrap();

    let mut data = BTreeMap::<String, Value>::new();

    let restaurant = state.restaurant(restaurant_id).unwrap();
    let menus = state.menus_for_restaurant(restaurant_id).unwrap();

    data.insert("restaurant".to_string(), value::to_value(&restaurant));
    data.insert("menus".to_string(), value::to_value(&menus));

    Ok(Response::with((status::Ok, Template::new("restaurant", data))))
}

fn ingest(req: &mut Request) -> IronResult<Response> {
    let restaurant_id = req.extensions.get::<Router>().unwrap()
        .find("id").unwrap()
        .parse::<i32>().unwrap();

    match req.get::<bodyparser::Struct<takedown::Menu>>() {
        Ok(Some(new_menu)) => {
            let state = req.extensions.get::<StateContainer>().unwrap().0.lock().unwrap();
            state.ingest_menu(restaurant_id, &new_menu).unwrap();

            Ok(Response::with(status::Ok))
        }
        Ok(None) => Ok(Response::with((status::BadRequest, "Missing body"))),
        Err(err) => Ok(Response::with((status::BadRequest, format!("{:?}", err)))),
    }
}

fn menu(req: &mut Request) -> IronResult<Response> {
    use self::serde_json::value::{self, Value};
    let state = req.extensions.get::<StateContainer>().unwrap().0.lock().unwrap();

    let menu_id = req.extensions.get::<Router>().unwrap()
        .find("id").unwrap()
        .parse::<i32>().unwrap();

    let mut data = BTreeMap::<String, Value>::new();

    let menu = state.menu(menu_id).unwrap();

    data.insert("menu".to_string(), value::to_value(&menu));

    Ok(Response::with((status::Ok, Template::new("menu", data))))
}

pub fn run(state: state::State, bind: &str, base_url: String, slack_token: Option<String>) -> Result<(), Error> {
    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./templates/", ".hbs")));
    hbse.reload()?;

    let mut router = Router::new();
    router.get("/", index, "index");
    router.post("/restaurant/", create_restaurant, "create_restaurant");
    router.get("/restaurant/:id", restaurant, "restaurant");
    router.post("/restaurant/:id", ingest, "ingest");
    router.get("/menu/:id", menu, "menu");
    router.post("/slack",
        move |req: &mut Request| {
            slack::slack(&slack_token.as_ref().map(String::as_ref), req)
        },
        "slack");

    let mut chain = Chain::new(router);
    chain.link_before(StateContainer(Arc::new(Mutex::new(state))));
    chain.link_before(EnvContainer(Arc::new(Env{ base_url: base_url })));
    chain.link_after(hbse);

    let listening = Iron::new(chain).http(bind).map_err(|_| Error::Bummer)?;
    println!("Listening to {:?}", &listening.socket);
    drop(listening); // Will implicitly block and keep handling requests

    Ok(())
}
