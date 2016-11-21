extern crate handlebars_iron;
extern crate iron;
extern crate router;
extern crate serde_json;

use std::sync::Mutex;
use std::collections::BTreeMap;
use state;

use self::handlebars_iron::{Template, HandlebarsEngine, DirectorySource};
use self::iron::prelude::*;
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

    let mut resp = Response::new();

    let mut data = BTreeMap::<String, Value>::new();

    // TODO Understand error handling with Iron and use ? here:
    let resturants = state.resturants().unwrap();

    data.insert("resturants".to_string(), value::to_value(&resturants));

    resp.set_mut(Template::new("index", data));
    Ok(resp)
}

pub fn run(state: state::State) -> Result<(), Error> {
    let shared_state = Mutex::new(state);

    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./templates/", ".hbs")));
    hbse.reload()?;

    let mut router = Router::new();
    router.get("/", move |req: &mut Request| index(&mut shared_state.lock().unwrap(), req), "index");

    let mut chain = Chain::new(router);
    chain.link_after(hbse);

    let listening = Iron::new(chain).http("localhost:3000").map_err(|_| Error::Bummer)?;
    println!("Listening to {:?}", &listening.socket);
    drop(listening); // Will implicitly block and keep handling requests

    Ok(())
}
