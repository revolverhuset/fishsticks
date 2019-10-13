extern crate bodyparser;
extern crate iron;
extern crate router;
extern crate serde_json;
extern crate urlencoded;

use models::{self, MenuId, RestaurantId};
use slack;
use state;
use std::fmt::Display;
use std::sync::{Arc, Mutex};

use self::iron::prelude::*;
use self::iron::{status, typemap, BeforeMiddleware};
use self::router::Router;
use self::urlencoded::UrlEncodedBody;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        IronHttp(err: iron::error::HttpError) { from() }
    }
}

impl From<state::Error> for iron::IronError {
    fn from(err: state::Error) -> iron::IronError {
        let msg = format!("{:?}", &err);
        iron::IronError::new(err, (status::InternalServerError, msg))
    }
}

#[derive(BartDisplay)]
#[template = "templates/layout.html"]
struct Layout<'a> {
    body: &'a Display,
}

impl<'a> Layout<'a> {
    fn new(body: &'a dyn Display) -> Layout<'a> {
        Layout { body: body }
    }
}

impl<'a> iron::modifier::Modifier<iron::Response> for Layout<'a> {
    fn modify(self, response: &mut iron::Response) {
        response.headers.set(iron::headers::ContentType::html());
        format!("{}", &self).modify(response);
    }
}

#[derive(Clone)]
pub struct StateContainer(pub Arc<Mutex<state::State>>);

impl typemap::Key for StateContainer {
    type Value = StateContainer;
}

impl BeforeMiddleware for StateContainer {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<StateContainer>(self.clone());
        Ok(())
    }
}

pub struct Env {
    pub base_url: String,
    pub maybe_sharebill_url: Option<String>,
    pub sharebill_cookies: Vec<String>,
}

#[derive(Clone)]
pub struct EnvContainer(pub Arc<Env>);

impl typemap::Key for EnvContainer {
    type Value = EnvContainer;
}

impl BeforeMiddleware for EnvContainer {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<EnvContainer>(self.clone());
        Ok(())
    }
}

fn index(req: &mut Request) -> IronResult<Response> {
    let state = req
        .extensions
        .get::<StateContainer>()
        .unwrap()
        .0
        .lock()
        .unwrap();

    #[derive(BartDisplay)]
    #[template = "templates/index.html"]
    struct Index {
        restaurants: Vec<models::Restaurant>,
    }

    Ok(Response::with((
        status::Ok,
        Layout::new(&Index {
            restaurants: state.restaurants()?,
        }),
    )))
}

fn create_restaurant(req: &mut Request) -> IronResult<Response> {
    use self::iron::headers::Location;
    use self::iron::modifiers::Header;
    let hashmap = req.get::<UrlEncodedBody>().unwrap();

    let name: Option<&str> = hashmap
        .get("name")
        .and_then(|x| x.get(0))
        .map(String::as_ref);

    if name.is_none() {
        return Ok(Response::with(status::BadRequest));
    }
    let name = name.unwrap();

    let state = req
        .extensions
        .get::<StateContainer>()
        .unwrap()
        .0
        .lock()
        .unwrap();
    let ref env = req.extensions.get::<EnvContainer>().unwrap().0;
    let id = match state.create_restaurant(name) {
        Ok(id) => id,
        Err(_) => return Ok(Response::with((status::InternalServerError, "Error!"))),
    };

    let created_url = format!("{}restaurant/{}", &env.base_url, i32::from(id));

    Ok(Response::with((
        status::Created,
        format!("New restaurant at {}", &created_url),
        Header(Location(created_url)),
    )))
}

fn restaurant(req: &mut Request) -> IronResult<Response> {
    let state = req
        .extensions
        .get::<StateContainer>()
        .unwrap()
        .0
        .lock()
        .unwrap();

    let restaurant_id: RestaurantId = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("id")
        .unwrap()
        .parse::<i32>()
        .unwrap()
        .into();

    #[derive(BartDisplay)]
    #[template = "templates/restaurant.html"]
    struct Restaurant {
        restaurant: models::Restaurant,
        menus: Vec<models::Menu>,
    }

    Ok(Response::with((
        status::Ok,
        Layout::new(&Restaurant {
            restaurant: state.restaurant(restaurant_id)?.unwrap(),
            menus: state.menus_for_restaurant(restaurant_id)?,
        }),
    )))
}

fn ingest(req: &mut Request) -> IronResult<Response> {
    let restaurant_id: RestaurantId = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("id")
        .unwrap()
        .parse::<i32>()
        .unwrap()
        .into();

    match req
        .get::<bodyparser::Raw>()
        .map(|x| x.map(|x| serde_json::from_str(&x)))
    {
        Ok(Some(Ok(new_menu))) => {
            println!("{:?}", &new_menu);

            let state = req
                .extensions
                .get::<StateContainer>()
                .unwrap()
                .0
                .lock()
                .unwrap();
            state.ingest_menu(restaurant_id, &new_menu)?;

            Ok(Response::with(status::Ok))
        }
        Ok(Some(Err(err))) => Ok(Response::with((status::BadRequest, format!("{:?}", err)))),
        Ok(None) => Ok(Response::with((status::BadRequest, "Missing body"))),
        Err(err) => Ok(Response::with((status::BadRequest, format!("{:?}", err)))),
    }
}

fn menu(req: &mut Request) -> IronResult<Response> {
    let state = req
        .extensions
        .get::<StateContainer>()
        .unwrap()
        .0
        .lock()
        .unwrap();

    let menu_id: MenuId = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("id")
        .unwrap()
        .parse::<i32>()
        .unwrap()
        .into();

    #[derive(BartDisplay)]
    #[template = "templates/menu.html"]
    struct Menu {
        menu: Vec<models::MenuItem>,
    }

    Ok(Response::with((
        status::Ok,
        Layout::new(&Menu {
            menu: state.menu(menu_id)?,
        }),
    )))
}

pub fn run(
    state: Arc<Mutex<state::State>>,
    bind: &str,
    base_url: String,
    slack_token: Option<String>,
    sharebill_url: Option<String>,
    sharebill_cookies: Vec<String>,
) -> Result<(), Error> {
    let mut router = Router::new();
    router.get("/", index, "index");
    router.post("/restaurant/", create_restaurant, "create_restaurant");
    router.get("/restaurant/:id", restaurant, "restaurant");
    router.post("/restaurant/:id", ingest, "ingest");
    router.get("/menu/:id", menu, "menu");
    router.post(
        "/slack",
        move |req: &mut Request| slack::slack(&slack_token.as_ref().map(String::as_ref), req),
        "slack",
    );

    let mut chain = Chain::new(router);
    chain.link_before(StateContainer(state));
    chain.link_before(EnvContainer(Arc::new(Env {
        base_url: base_url,
        maybe_sharebill_url: sharebill_url,
        sharebill_cookies: sharebill_cookies,
    })));

    let listening = Iron::new(chain).http(bind)?;
    println!("Listening to {:?}", &listening.socket);
    drop(listening); // Will implicitly block and keep handling requests

    Ok(())
}
