extern crate handlebars_iron;
extern crate iron;
extern crate router;

use self::iron::prelude::*;
use self::router::Router;
use self::handlebars_iron::{Template, HandlebarsEngine, DirectorySource};

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Bummer
        Handlebars(err: handlebars_iron::SourceError) { from() }
    }
}

fn hello_world(_: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();

    let data = ();
    resp.set_mut(Template::new("index", data));
    Ok(resp)
}

pub fn run() -> Result<(), Error> {
    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./templates/", ".hbs")));
    hbse.reload()?;

    let mut router = Router::new();
    router.get("/", hello_world, "hello");

    let mut chain = Chain::new(router);
    chain.link_after(hbse);

    let listening = Iron::new(chain).http("localhost:3000").map_err(|_| Error::Bummer)?;
    println!("Listening to {:?}", &listening.socket);
    drop(listening); // Will implicitly block and keep handling requests

    Ok(())
}
