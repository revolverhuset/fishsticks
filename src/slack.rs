extern crate iron;
extern crate itertools;
extern crate num;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate sharebill;
extern crate time;
extern crate urlencoded;
extern crate uuid;

use state;
use std;
use web;
use words::*;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::headers::ContentType;
use self::iron::modifiers::Header;
use self::itertools::*;
use self::urlencoded::UrlEncodedBody;

quick_error! {
    #[derive(Debug)]
    enum Error {
        StateError(err: state::Error) { from() }
        UrlDecodingError(err: urlencoded::UrlDecodingError) { from() }
        PoisonError
        InputError { from(std::num::ParseFloatError) }
        InvalidSlackToken
        MissingAssociation(slack_name: String)
        SerdeJson(err: serde_json::Error) { from() }
        UnexpectedStatus(status: reqwest::StatusCode)
        NotFound
        MissingConfig(config_path: &'static str)
        FormatError(err: std::fmt::Error) { from() }
        ReqwestError(err: reqwest::Error) { from() }
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
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
            base_url, i32::from(menu.id), &restaurant.name),
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
        ref items if items.len() > 1 => {
            use std::fmt::Write;
            let mut buf = String::new();

            writeln!(&mut buf, ":information_desk_person: The best matches I found for {:?} are:\n", &args)?;
            for item in items[..4].iter() {
                writeln!(&mut buf, " - {}. {}", item.number, item.name)?;
            }

            Ok(SlackResponse {
                response_type: ResponseType::Ephemeral,
                text: buf,
                unfurl_links: false,
            })
        },
        ref mut items if items.len() == 1 => {
            let menu_item = items.pop().expect("Guaranteed because of match arm");
            Ok(SlackResponse {
                response_type: ResponseType::Ephemeral,
                text: format!(":information_desk_person: That query matches the {} \
                    {} {}. {}", adjective(), noun(), &menu_item.number, &menu_item.name),
                unfurl_links: false,
            })
        },
        _ => Ok(SlackResponse {
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

    match state.query_menu(open_order.menu, &query)?.pop() {
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
    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;
    let items = state.items_in_order(open_order.id)?;

    use std::fmt::Write;
    let mut buf = String::new();

    for (person_name, items) in
        items.into_iter()
            .group_by(|&(_, ref order_item)| order_item.person_name.clone()).into_iter()
    {
        writeln!(&mut buf, "{}:", person_name)?;
        for (menu_item, _) in items {
            writeln!(&mut buf, " - {}. {}", menu_item.number, menu_item.name)?;
        }
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

fn cmd_sharebill(state_mutex: &Mutex<state::State>, args: &str, user_name: &str, sharebill_url: &str) -> Result<SlackResponse, Error> {
    use std::collections::HashMap;
    use sharebill::Rational;
    use self::num::Zero;

    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;

    let description = format!("{}",
        state.restaurant(
            state.menu_object(open_order.menu)?
                .ok_or(Error::NotFound)?
                .restaurant
        )?
        .ok_or(Error::NotFound)?
        .name
    );

    let associations = state.all_associations()?.into_iter()
        .map(|x| (x.slack_name, x.sharebill_account))
        .collect::<HashMap<_, _>>();

    let items = state.items_in_order(open_order.id)?;

    let persons = Rational::from(items.iter()
        .group_by(|&&(_, ref order_item)| order_item.person_name.clone()).into_iter()
        .count());

    let overhead = Rational::from_cents(open_order.overhead_in_cents);
    let overhead_per_person = overhead / persons;

    let slack_debits = items.into_iter()
        .group_by(|&(_, ref order_item)| order_item.person_name.clone()).into_iter()
        .map(|(person_name, items)| {
            let food = items.into_iter()
                .map(|(menu_item, _)| Rational::from_cents(menu_item.price_in_cents))
                .fold(Rational::zero(), |acc, x| acc + x);

            (person_name, food + &overhead_per_person)
        })
        .collect::<Vec<_>>();

    // Associations are deliberately used to bill orders by different
    // people to the same accuont. This is handled below:
    let mut debits = HashMap::<String, Rational>::new();
    for (slack_name, value) in slack_debits {
        let account = associations.get(&slack_name).ok_or(Error::MissingAssociation(slack_name))?;
        let entry = debits.entry(account.clone()).or_insert_with(Rational::zero);
        *entry = &*entry + value;
    }

    let credit_account = match args.len() {
        0 => associations.get(user_name).map(|x| x as &str),
        _ => Some(args)
    }.ok_or(Error::MissingAssociation(user_name.to_owned()))?;

    let total = debits.values().fold(Rational::zero(), |acc, x| acc + x);

    let mut credits = HashMap::<String, Rational>::new();
    credits.insert(credit_account.to_owned(), total);

    let post = sharebill::models::Post {
        meta: sharebill::models::Meta {
            description: description,
            timestamp: time::now_utc()
        },
        transaction: sharebill::models::Transaction {
            debits: debits,
            credits: credits
        }
    };

    let target_url = format!("{}post/{}", &sharebill_url, &uuid::Uuid::new_v4());

    let res = reqwest::Client::new()?
        .request(reqwest::Method::Put, &target_url)
        .json(&post)
        .send()?;

    if res.status() != &reqwest::StatusCode::Created {
        return Err(Error::UnexpectedStatus(res.status().clone()));
    }

    state.close_current_order()?;

    Ok(SlackResponse {
        response_type: ResponseType::InChannel,
        text: format!(":money_with_wings: Posted to <{}|Sharebill> and closed order :heavy_check_mark:", target_url),
        unfurl_links: false,
    })
}

fn cmd_overhead(state_mutex: &Mutex<state::State>, args: &str) -> Result<SlackResponse, Error> {
    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;

    if args.len() == 0 {
        Ok(SlackResponse {
            response_type: ResponseType::Ephemeral,
            text: format!(
                ":information_desk_person: Overhead is set to {}.{:02}",
                open_order.overhead_in_cents / 100, open_order.overhead_in_cents % 100),
            unfurl_links: false,
        })
    } else {
        let overhead = args.parse::<f64>()?;
        let overhead_in_cents = (overhead * 100.0) as i32;

        state.set_overhead(open_order.id, overhead_in_cents)?;

        Ok(SlackResponse {
            response_type: ResponseType::InChannel,
            text: format!(
                ":information_desk_person: Overhead is now {}.{:02}",
                overhead_in_cents / 100, overhead_in_cents % 100),
            unfurl_links: false,
        })
    }
}

fn slack_core(
    maybe_slack_token: &Option<&str>,
    maybe_sharebill_url: &Option<&str>,
    req: &mut Request,
) ->
    Result<SlackResponse, Error>
{
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
                    overhead [VALUE]\n    Get/set overhead (delivery cost, gratuity, etc) for current order\n\
                    restaurants\n    List known restaurants\n\
                    search QUERY\n    See what matches QUERY in the menu\n\
                    sharebill [CREDIT_ACCOUNT]\n    Post order to Sharebill\n\
                    summary\n    See the current order\n\
                    ".to_owned(),
                unfurl_links: false,
            }),
        "associate" => cmd_associate(&state_mutex, args, user_name),
        "closeorder" => cmd_closeorder(&state_mutex, args),
        "openorder" => cmd_openorder(&state_mutex, args, &env.base_url),
        "order" => cmd_order(&state_mutex, args, user_name),
        "overhead" => cmd_overhead(&state_mutex, args),
        "restaurants" => cmd_restaurants(&state_mutex, args),
        "search" => cmd_search(&state_mutex, args),
        "sharebill" => cmd_sharebill(&state_mutex, args, user_name,
            maybe_sharebill_url.ok_or(Error::MissingConfig("web.sharebill_url"))?),
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

pub fn slack(slack_token: &Option<&str>, sharebill_url: &Option<&str>, req: &mut Request) -> IronResult<Response> {
    match slack_core(slack_token, sharebill_url, req) {
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
