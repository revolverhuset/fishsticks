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

fn adjective() -> &'static str {
    const ADJECTIVES: &'static [&'static str] = &[
        "delicious",
        "tasty",
        "yummy",
        "edible",
        "awesome",
        "sick",
    ];

    use self::rand::Rng;
    rand::thread_rng().choose(ADJECTIVES).unwrap()
}

fn noun() -> &'static str {
    const NOUNS: &'static [&'static str] = &[
        "treat",
        "edible",
        "food",
        "fishstick",
    ];

    use self::rand::Rng;
    rand::thread_rng().choose(NOUNS).unwrap()
}

fn affirm() -> &'static str {
    const STRS: &'static [&'static str] = &[
        "I'll get you",
        "You're getting",
        "I'mma get you",
        "I'm taking that down as",
    ];

    use self::rand::Rng;
    rand::thread_rng().choose(STRS).unwrap()
}

quick_error! {
    #[derive(Debug)]
    enum Error {
        StateError(err: state::Error) { from() }
        UrlDecodingError(err: urlencoded::UrlDecodingError) { from() }
        PoisonError
        InputError
        InvalidSlackToken
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
    unfurl_links: bool,
}

use std::sync::Mutex;

fn cmd_restaurants(state_mutex: &Mutex<state::State>, _args: &str) -> Result<SlackResponse, Error> {
    let state = state_mutex.lock()?;
    let restaurants = state.restaurants()?.into_iter()
        .map(|x| x.name)
        .collect::<Vec<_>>()
        .join(", ");

    Ok(SlackResponse {
        response_type: ResponseType::Ephemeral,
        text: format!("I know of these restaurants: {}",
            &restaurants),
        unfurl_links: false,
    })
}

fn cmd_openorder(state_mutex: &Mutex<state::State>, args: &str, base_url: &str) -> Result<SlackResponse, Error> {
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
                unfurl_links: false,
            })
        },
    };

    let menu = state.current_menu_for_restaurant(restaurant.id)?;

    let _new_order = state.create_order(menu.id)?;

    Ok(SlackResponse {
        response_type: ResponseType::InChannel,
        text: format!(":bell: Now taking orders from the \
            <{}menu/{}|{} menu> :memo:",
            base_url, menu.id, &restaurant.name),
        unfurl_links: false,
    })
}

fn cmd_closeorder(state_mutex: &Mutex<state::State>, _args: &str) -> Result<SlackResponse, Error> {
    let state = state_mutex.lock()?;

    state.close_current_order()?;

    Ok(SlackResponse {
        response_type: ResponseType::InChannel,
        text: format!("No longer taking orders"),
        unfurl_links: false,
    })
}

fn cmd_search(state_mutex: &Mutex<state::State>, args: &str) -> Result<SlackResponse, Error> {
    let query = state::Query::interpret_string(&args);

    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;

    match state.query_menu(open_order.menu, &query)? {
        Some(menu_item) => Ok(SlackResponse {
            response_type: ResponseType::Ephemeral,
            text: format!(":information_desk_person: That query matches the {} \
                {} {}. {}", adjective(), noun(), &menu_item.number, &menu_item.name),
            unfurl_links: false,
        }),
        None => Ok(SlackResponse {
            response_type: ResponseType::Ephemeral,
            text: format!(":person_frowning: I found no matches for {:?}", &args),
            unfurl_links: false,
        }),
    }
}

fn cmd_order(state_mutex: &Mutex<state::State>, args: &str, user_name: &str) -> Result<SlackResponse, Error> {
    let query = state::Query::interpret_string(&args);

    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;

    match state.query_menu(open_order.menu, &query)? {
        Some(menu_item) => {
            state.add_order_item(open_order.id, user_name, menu_item.id)?;

            Ok(SlackResponse {
                response_type: ResponseType::InChannel,
                text: format!(":information_desk_person: {} the {} {} {}. {}",
                    affirm(), adjective(), noun(), &menu_item.number, &menu_item.name),
                unfurl_links: false,
            })
        },
        None => Ok(SlackResponse {
            response_type: ResponseType::Ephemeral,
            text: format!(":person_frowning: I found no matches for {:?}", &args),
            unfurl_links: false,
        }),
    }
}

fn cmd_summary(state_mutex: &Mutex<state::State>, _args: &str) -> Result<SlackResponse, Error> {
    use std::fmt::Write;

    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;
    let items = state.items_in_order(open_order.id)?;

    let mut buf = String::new();

    write!(&mut buf, ":raising_hand::memo: I've got:\n").unwrap();
    for (menu_item, order_item) in items {
        write!(&mut buf,
            "{}: {}. {}\n",
            order_item.person_name, menu_item.number, menu_item.name
        ).unwrap();
    }

    Ok(SlackResponse {
        response_type: ResponseType::Ephemeral,
        text: buf,
        unfurl_links: false,
    })
}

fn cmd_associate(state_mutex: &Mutex<state::State>, args: &str, user_name: &str) -> Result<SlackResponse, Error> {
    if args.len() == 0 {
        let state = state_mutex.lock()?;
        let associations = state.all_associations()?.into_iter()
            .map(|x| format!("{} \u{2192} {}", &x.slack_name, &x.sharebill_account))
            .collect::<Vec<_>>()
            .join("\n    ");

        Ok(SlackResponse {
            response_type: ResponseType::Ephemeral,
            text: format!("I have the following mappings from slack names to sharebill accounts:\n    {}",
                &associations),
            unfurl_links: false,
        })
    } else {
        let split = args.split_whitespace().collect::<Vec<_>>();
        let (slack_name, sharebill_account) = match split.len() {
            1 => (user_name, split[0]),
            2 => (split[0], split[1]),
            _ => return Err(Error::InputError),
        };

        let state = state_mutex.lock()?;
        state.set_association(slack_name, sharebill_account)?;

        Ok(SlackResponse {
            response_type: ResponseType::Ephemeral,
            text: format!("Billing orders by {} to account {}. Got it :+1:",
                slack_name, sharebill_account),
            unfurl_links: false,
        })
    }
}

fn slack_core(maybe_slack_token: &Option<&str>, req: &mut Request) -> Result<SlackResponse, Error> {
    let hashmap = req.get::<UrlEncodedBody>()?;

    println!("Parsed GET request query string:\n {:?}", hashmap);

    if let &Some(slack_token) = maybe_slack_token {
        let given_token =
            hashmap.get("token")
                .and_then(|tokens| tokens.get(0))
                .map(String::as_ref);

        if given_token != Some(slack_token) {
            return Err(Error::InvalidSlackToken);
        }
    }

    if hashmap.contains_key("sslcheck") {
        return Ok(SlackResponse {
            response_type: ResponseType::Ephemeral,
            text: String::new(),
            unfurl_links: false,
        });
    }

    let ref state_mutex = req.extensions.get::<web::StateContainer>().unwrap().0;
    let ref env = req.extensions.get::<web::EnvContainer>().unwrap().0;

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
                    associate [SLACK_NAME] SHAREBILL_ACCOUNT\n    Associate the given slack name (defaults to your name) with the given sharebill account\n\
                    associate\n    Display all slack name-sharebill account associations\n\
                    closeorder\n    Close the current order\n\
                    help\n    This help\n\
                    openorder RESTAURANT\n    Start a new order from the given restaurant\n\
                    order QUERY\n    Order whatever matches QUERY in the menu\n\
                    restaurants\n    List known restaurants\n\
                    search QUERY\n    See what matches QUERY in the menu\n\
                    summary\n    See the current order\n\
                    ".to_owned(),
                unfurl_links: false,
            }),
        "associate" => cmd_associate(&state_mutex, args, user_name),
        "closeorder" => cmd_closeorder(&state_mutex, args),
        "openorder" => cmd_openorder(&state_mutex, args, &env.base_url),
        "order" => cmd_order(&state_mutex, args, user_name),
        "restaurants" => cmd_restaurants(&state_mutex, args),
        "search" => cmd_search(&state_mutex, args),
        "summary" => cmd_summary(&state_mutex, args),
        _ =>
            Ok(SlackResponse {
                response_type: ResponseType::Ephemeral,
                text: format!(":confused: Oh man! I don't understand /ffs {} {}\n\
                    Try /ffs help", &cmd, &args),
                unfurl_links: false,
            }),
    }
}

pub fn slack(slack_token: &Option<&str>, req: &mut Request) -> IronResult<Response> {
    match slack_core(slack_token, req) {
        Ok(response) => Ok(Response::with((
            status::Ok,
            serde_json::to_string(&response).unwrap(),
            Header(ContentType::json()),
        ))),
        Err(err) => Ok(Response::with((
            status::InternalServerError,
            serde_json::to_string(&SlackResponse {
                response_type: ResponseType::Ephemeral,
                text: format!(":no_good: {:?}", &err),
                unfurl_links: false,
            }).unwrap(),
            Header(ContentType::json()),
        )))
    }
}
