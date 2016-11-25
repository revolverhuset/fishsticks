extern crate iron;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate urlencoded;

use state;
use std;
use web;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::headers::ContentType;
use self::iron::modifiers::Header;
use self::urlencoded::UrlEncodedBody;

const ADJECTIVES: &'static [&'static str] = &[
    "delicious",
    "tasty",
    "yummy",
    "edible",
    "awesome",
    "sick",
];

const NOUNS: &'static [&'static str] = &[
    "treat",
    "edible",
    "food",
    "fishstick",
];

fn adjective() -> &'static str {
    use self::rand::Rng;
    rand::thread_rng().choose(ADJECTIVES).unwrap()
}

fn noun() -> &'static str {
    use self::rand::Rng;
    rand::thread_rng().choose(NOUNS).unwrap()
}

quick_error! {
    #[derive(Debug)]
    enum Error {
        StateError(err: state::Error) { from() }
        UrlDecodingError(err: urlencoded::UrlDecodingError) { from() }
        PoisonError
    }
}

impl<T> std::convert::From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::PoisonError
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
struct SlackResponse {
    response_type: ResponseType,
    text: String,
}

fn slack_core(req: &mut Request) -> Result<SlackResponse, Error> {
    let hashmap = req.get::<UrlEncodedBody>()?;

    println!("Parsed GET request query string:\n {:?}", hashmap);

    if hashmap.contains_key("sslcheck") {
        return Ok(SlackResponse {
            response_type: ResponseType::Ephemeral,
            text: String::new(),
        });
    }

    let ref state_mutex = req.extensions.get::<web::StateContainer>().unwrap().0;

    let text = &hashmap.get("text").unwrap()[0];
    let mut split = text.splitn(2, ' ');
    let cmd = split.next().unwrap();
    let args = split.next().unwrap_or("");

    let user_name = &hashmap.get("user_name").unwrap()[0];

    match cmd {
        "help" =>
            Ok(SlackResponse {
                response_type: ResponseType::Ephemeral,
                text: "USAGE: /ffs command args...\n\
                    /ffs help\n    This help\n\
                    /ffs openorder RESTAURANT\n    Start a new order from the given restaurant\n\
                    /ffs restaurants\n    List known restaurants\n\
                    /ffs search QUERY\n    Look for QUERY in the menu\n\
                    ".to_owned(),
            }),
        "restaurants" => {
            let state = state_mutex.lock()?;
            let restaurants = state.restaurants()?.into_iter()
                .map(|x| x.name)
                .collect::<Vec<_>>()
                .join(", ");

            Ok(SlackResponse {
                response_type: ResponseType::Ephemeral,
                text: format!("I know of these restaurants: {}",
                    &restaurants),
            })
        },
        "openorder" => {
            let state = state_mutex.lock()?;

            let restaurant = match state.restaurant_by_name(args)? {
                Some(resturant) => resturant,
                None => {
                    let restaurants = state.restaurants()?.into_iter()
                        .map(|x| x.name)
                        .collect::<Vec<_>>()
                        .join(", ");

                    return Ok(SlackResponse {
                        response_type: ResponseType::Ephemeral,
                        text: format!("Usage: /ffs openorder RESTAURANT\n\
                            I know of these restaurants: {}",
                            &restaurants),
                    })
                },
            };

            let _new_order = state.create_order(restaurant.id)?;

            Ok(SlackResponse {
                response_type: ResponseType::InChannel,
                text: format!(":bell: Now taking orders from the {} menu :memo:",
                    &restaurant.name),
            })
        },
        "closeorder" => {
            let state = state_mutex.lock()?;

            state.close_current_order()?;

            Ok(SlackResponse {
                response_type: ResponseType::InChannel,
                text: format!("No longer taking orders"),
            })
        },
        "search" => {
            let query = state::Query::interpret_string(&args);

            let state = state_mutex.lock()?;

            let open_order = state.demand_open_order()?;

            match state.query_menu(open_order.restaurant, &query)? {
                Some(menu_item) => Ok(SlackResponse {
                    response_type: ResponseType::Ephemeral,
                    text: format!(":information_desk_person: That query matches the {} \
                        {} {}. {}", adjective(), noun(), &menu_item.id, &menu_item.name),
                }),
                None => Ok(SlackResponse {
                    response_type: ResponseType::Ephemeral,
                    text: format!(":person_frowning: I found no matches for {:?}", &args),
                }),
            }
        },
        _ =>
            Ok(SlackResponse {
                response_type: ResponseType::Ephemeral,
                text: format!(":confused: Aw, shucks, I don't understand /ffs {} {}\n\
                    Try /ffs help", &cmd, &args),
            }),
    }
}

pub fn slack(req: &mut Request) -> IronResult<Response> {
    match slack_core(req) {
        Ok(response) => Ok(Response::with((
            status::Ok,
            serde_json::to_string(&response).unwrap(),
            Header(ContentType::json()),
        ))),
        Err(err) => Ok(Response::with((
            status::InternalServerError,
            serde_json::to_string(&SlackResponse {
                response_type: ResponseType::Ephemeral,
                text: format!("{:?}", &err),
            }).unwrap(),
            Header(ContentType::json()),
        )))
    }
}
