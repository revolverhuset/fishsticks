extern crate iron;
extern crate serde;
extern crate serde_json;
extern crate urlencoded;

use web;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::headers::ContentType;
use self::iron::modifiers::Header;
use self::urlencoded::UrlEncodedBody;

enum ResponseType {
    Ephemeral,
    InChannel,
}

impl serde::Serialize for ResponseType {
    fn serialize<S: serde::Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_str(match *self {
            ResponseType::Ephemeral => "ephemeral",
            ResponseType::InChannel => "in_channel",
        })
    }
}

#[derive(Serialize)]
struct SlackResponse<'a> {
    response_type: ResponseType,
    text: &'a str,
}

pub fn slack(req: &mut Request) -> IronResult<Response> {
    let hashmap = req.get::<UrlEncodedBody>().unwrap();

    println!("Parsed GET request query string:\n {:?}", hashmap);

    if hashmap.contains_key("sslcheck") {
        return Ok(Response::with(status::Ok));
    }

    let ref state_mutex = req.extensions.get::<web::StateContainer>().unwrap().0;

    let text = &hashmap.get("text").unwrap()[0];
    let mut split = text.splitn(2, ' ');
    let cmd = split.next().unwrap();
    let args = split.next();

    match cmd {
        "help" =>
            Ok(Response::with((
                status::Ok,
                serde_json::to_string(&SlackResponse {
                    response_type: ResponseType::Ephemeral,
                    text: "USAGE: /ffs command args...\n\
                        /ffs help\n\tThis help\n\
                        /ffs restaurants\n\tList known restaurants\n\
                        ",
                }).unwrap(),
                Header(ContentType::json()),
            ))),
        "restaurants" => {
            let state = state_mutex.lock().unwrap();

            let restaurants = state.restaurants().unwrap().into_iter()
                .map(|x| x.name)
                .collect::<Vec<_>>()
                .join(", ");

            Ok(Response::with((
                status::Ok,
                serde_json::to_string(&SlackResponse {
                    response_type: ResponseType::Ephemeral,
                    text: &format!("I know of these restaurants: {}",
                        &restaurants),
                }).unwrap(),
                Header(ContentType::json()),
            )))
        },
        _ =>
            Ok(Response::with((
                status::Ok,
                serde_json::to_string(&SlackResponse {
                    response_type: ResponseType::Ephemeral,
                    text: &format!("Aw, shucks, I don't understand /ffs {} {}\n\
                        Try /ffs help", &cmd, &args.unwrap_or("")),
                }).unwrap(),
                Header(ContentType::json()),
            ))),
    }
}
