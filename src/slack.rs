extern crate iron;
extern crate serde;
extern crate serde_json;
extern crate urlencoded;

use state;
use std::convert;
use web;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::headers::ContentType;
use self::iron::modifiers::Header;
use self::urlencoded::UrlEncodedBody;

impl convert::From<state::Error> for iron::IronError {
    fn from(err: state::Error) -> Self {
        let response = serde_json::to_string(&SlackResponse {
            response_type: ResponseType::Ephemeral,
            text: &format!("{:?}", &err),
        }).unwrap();

        iron::IronError::new(
            err,
            (
                status::InternalServerError,
                response,
                Header(ContentType::json()),
            )
        )
    }
}

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
    let args = split.next().unwrap_or("");

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
        "openorder" => {
            let state = state_mutex.lock().unwrap();

            let restaurant = match state.restaurant_by_name(args)? {
                Some(resturant) => resturant,
                None => {
                    let restaurants = state.restaurants().unwrap().into_iter()
                        .map(|x| x.name)
                        .collect::<Vec<_>>()
                        .join(", ");

                    return Ok(Response::with((
                        status::Ok,
                        serde_json::to_string(&SlackResponse {
                            response_type: ResponseType::Ephemeral,
                            text: &format!("Usage: /ffs openorder RESTAURANT\n\
                                I know of these restaurants: {}",
                                &restaurants),
                        }).unwrap(),
                        Header(ContentType::json()),
                    )))
                },
            };

            let _new_order = state.create_order(restaurant.id)?;

            Ok(Response::with((
                status::Ok,
                serde_json::to_string(&SlackResponse {
                    response_type: ResponseType::InChannel,
                    text: &format!(":bell: Now taking orders from the {} menu :memo:",
                        &restaurant.name),
                }).unwrap(),
                Header(ContentType::json()),
            )))
        },
        _ =>
            Ok(Response::with((
                status::Ok,
                serde_json::to_string(&SlackResponse {
                    response_type: ResponseType::Ephemeral,
                    text: &format!(":confused: Aw, shucks, I don't understand /ffs {} {}\n\
                        Try /ffs help", &cmd, &args),
                }).unwrap(),
                Header(ContentType::json()),
            ))),
    }
}
