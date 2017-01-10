extern crate bodyparser;
extern crate iron;
extern crate router;
extern crate urlencoded;

use models::{RestaurantId, MenuId};
use std::sync::{Arc, Mutex};
use slack;
use state;
use takedown;

use self::iron::headers::ContentType;
use self::iron::modifiers::Header;
use self::iron::prelude::*;
use self::iron::{status, typemap, BeforeMiddleware};
use self::router::Router;
use self::urlencoded::UrlEncodedBody;

// TODO Understand error handling with Iron
quick_error! {
    #[derive(Debug)]
    pub enum Error {
        IronHttp(err: iron::error::HttpError) { from() }
    }
}

mod template {
    use models;

    #[derive(BartDisplay)]
    #[template = "templates/header.html"]
    pub struct Header;

    #[derive(BartDisplay)]
    #[template = "templates/footer.html"]
    pub struct Footer;

    #[derive(BartDisplay)]
    #[template = "templates/index.html"]
    pub struct Index {
        pub header: Header,
        pub footer: Footer,
        pub restaurants: Vec<models::Restaurant>,
    }

    // Bart 0.0.1 can't handle field names with underscore, so
    // we have to map to another type without that.
    pub struct MenuItem {
        pub id: models::MenuItemId,
        pub menu: models::MenuId,
        pub number: i32,
        pub name: String,
        pub priceincents: i32,
    }
    impl From<models::MenuItem> for MenuItem {
        fn from(src: models::MenuItem) -> MenuItem {
            MenuItem {
                id: src.id,
                menu: src.menu,
                number: src.number,
                name: src.name,
                priceincents: src.price_in_cents,
            }
        }
    }

    #[derive(BartDisplay)]
    #[template = "templates/menu.html"]
    pub struct Menu {
        pub header: Header,
        pub footer: Footer,
        pub menu: Vec<MenuItem>,
    }

    #[derive(BartDisplay)]
    #[template = "templates/restaurant.html"]
    pub struct Restaurant {
        pub header: Header,
        pub footer: Footer,
        pub restaurant: models::Restaurant,
        pub menus: Vec<models::Menu>,
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
    let state = req.extensions.get::<StateContainer>().unwrap().0.lock().unwrap();

    Ok(Response::with((
        status::Ok,
        format!("{}", template::Index {
            header: template::Header,
            footer: template::Footer,
            restaurants: state.restaurants().unwrap()
        }),
        Header(ContentType::html()),
    )))
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

    let created_url = format!("{}restaurant/{}", &env.base_url, i32::from(id));

    Ok(Response::with((
        status::Created,
        format!("New restaurant at {}", &created_url),
        Header(Location(created_url)),
    )))
}

fn restaurant(req: &mut Request) -> IronResult<Response> {
    let state = req.extensions.get::<StateContainer>().unwrap().0.lock().unwrap();

    let restaurant_id : RestaurantId =
        req.extensions.get::<Router>().unwrap()
            .find("id").unwrap()
            .parse::<i32>().unwrap()
            .into();

    Ok(Response::with((
        status::Ok,
        format!("{}", template::Restaurant {
            header: template::Header,
            footer: template::Footer,
            restaurant: state.restaurant(restaurant_id).unwrap().unwrap(),
            menus: state.menus_for_restaurant(restaurant_id).unwrap(),
        }),
        Header(ContentType::html()),
    )))
}

fn ingest(req: &mut Request) -> IronResult<Response> {
    let restaurant_id : RestaurantId =
        req.extensions.get::<Router>().unwrap()
            .find("id").unwrap()
            .parse::<i32>().unwrap()
            .into();

    match req.get::<bodyparser::Struct<takedown::Menu>>() {
        Ok(Some(new_menu)) => {
            println!("{:?}", &new_menu);

            let state = req.extensions.get::<StateContainer>().unwrap().0.lock().unwrap();
            state.ingest_menu(restaurant_id, &new_menu).unwrap();

            Ok(Response::with(status::Ok))
        }
        Ok(None) => Ok(Response::with((status::BadRequest, "Missing body"))),
        Err(err) => Ok(Response::with((status::BadRequest, format!("{:?}", err)))),
    }
}

fn menu(req: &mut Request) -> IronResult<Response> {
    let state = req.extensions.get::<StateContainer>().unwrap().0.lock().unwrap();

    let menu_id: MenuId =
        req.extensions.get::<Router>().unwrap()
            .find("id").unwrap()
            .parse::<i32>().unwrap()
            .into();

    Ok(Response::with((
        status::Ok,
        format!("{}", template::Menu {
            header: template::Header,
            footer: template::Footer,
            menu: state.menu(menu_id).unwrap().into_iter().map(|x| x.into()).collect()
        }),
        Header(ContentType::html()),
    )))
}

pub fn run(
    state: state::State,
    bind: &str,
    base_url: String,
    slack_token: Option<String>,
    sharebill_url: Option<String>,
) ->
    Result<(), Error>
{
    let mut router = Router::new();
    router.get("/", index, "index");
    router.post("/restaurant/", create_restaurant, "create_restaurant");
    router.get("/restaurant/:id", restaurant, "restaurant");
    router.post("/restaurant/:id", ingest, "ingest");
    router.get("/menu/:id", menu, "menu");
    router.post("/slack",
        move |req: &mut Request| {
            slack::slack(
                &slack_token.as_ref().map(String::as_ref),
                &sharebill_url.as_ref().map(String::as_ref),
                req
            )
        },
        "slack");

    let mut chain = Chain::new(router);
    chain.link_before(StateContainer(Arc::new(Mutex::new(state))));
    chain.link_before(EnvContainer(Arc::new(Env{ base_url: base_url })));

    let listening = Iron::new(chain).http(bind)?;
    println!("Listening to {:?}", &listening.socket);
    drop(listening); // Will implicitly block and keep handling requests

    Ok(())
}
