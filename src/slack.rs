use cmd::{self, exec_cmd, CommandContext, Error};
use num::Zero;
use std::fmt::Write;
use web;
use words::*;

use iron::headers::ContentType;
use iron::modifiers::Header;
use iron::prelude::*;
use iron::status;
use urlencoded::UrlEncodedBody;

#[derive(Serialize)]
pub enum ResponseType {
    #[serde(rename = "ephemeral")]
    Ephemeral,

    #[serde(rename = "in_channel")]
    InChannel,
}

impl Default for ResponseType {
    fn default() -> ResponseType {
        ResponseType::Ephemeral
    }
}

#[derive(Serialize, Default)]
pub struct SlackResponse {
    pub response_type: ResponseType,
    pub text: String,
    pub unfurl_links: bool,
}

impl From<cmd::Response> for SlackResponse {
    fn from(src: cmd::Response) -> Self {
        use cmd::Response::*;
        match src {
            UnknownCommand { cmd, args } => SlackResponse {
                text: format!(
                    "😕 Oh man! I don't understand /ffs {} {}\n\
                    Try /ffs help",
                    cmd, args
                ),
                ..Default::default()
            },
            RepeatNoMatch => SlackResponse {
                text: format!("🙍 I found no matches for you"),
                ..Default::default()
            },
            OrderNoMatch { search_string } => SlackResponse {
                text: format!("🙍 I found no matches for {:?}", search_string),
                ..Default::default()
            },
            PlacedOrder { menu_items } => {
                if menu_items.len() == 1 {
                    let menu_item = &menu_items[0];

                    SlackResponse {
                        response_type: ResponseType::InChannel,
                        text: format!(
                            "💁 {} the {} {} {}. {}",
                            affirm(),
                            adjective(),
                            noun(),
                            &menu_item.number,
                            &menu_item.name
                        ),
                        ..Default::default()
                    }
                } else {
                    let summary = menu_items
                        .into_iter()
                        .map(|x| format!("{}. {}", x.number, x.name))
                        .collect::<Vec<_>>()
                        .join(", ");

                    SlackResponse {
                        response_type: ResponseType::InChannel,
                        text: format!(
                            "💁 {} the {} selection: {}",
                            affirm(),
                            adjective(),
                            summary
                        ),
                        ..Default::default()
                    }
                }
            }
            SearchResults { ref query, ref items } if items.len() > 1 => {
                let mut buf = String::new();

                writeln!(
                    &mut buf,
                    "💁 The best matches I found for {:?} are:\n",
                    query
                ).unwrap();
                for item in items[..4].iter() {
                    writeln!(&mut buf, " - {}. {}", item.number, item.name).unwrap();
                }

                SlackResponse {
                    text: buf,
                    ..Default::default()
                }
            }
            SearchResults { ref items, .. } if items.len() == 1 => {
                let menu_item = &items[0];
                SlackResponse {
                    text: format!(
                        "💁 That query matches the {} \
                        {} {}. {}",
                        adjective(),
                        noun(),
                        &menu_item.number,
                        &menu_item.name
                    ),
                    ..Default::default()
                }
            }
            SearchResults { query, .. } => SlackResponse {
                text: format!("🙍 I found no matches for {:?}", query),
                ..Default::default()
            },
            Restaurants { restaurants } => {
                let restaurants = restaurants
                    .into_iter()
                    .map(|x| x.name)
                    .collect::<Vec<_>>()
                    .join(", ");

                SlackResponse {
                    text: format!("I know of these restaurants: {}", &restaurants),
                    ..Default::default()
                }
            }
            RestaurantsNoMatch { restaurants } => {
                let restaurants = restaurants
                    .into_iter()
                    .map(|x| x.name)
                    .collect::<Vec<_>>()
                    .join(", ");

                SlackResponse {
                    text: format!(
                        "Usage: /ffs openorder RESTAURANT\n\
                         I know of these restaurants: {}",
                        &restaurants
                    ),
                    ..Default::default()
                }
            }
            OpenedOrder {
                menu_url,
                restaurant_name,
            } => SlackResponse {
                response_type: ResponseType::InChannel,
                text: format!(
                    "🔔 Now taking orders from the <{}|{} menu> 📝",
                    menu_url, restaurant_name
                ),
                ..Default::default()
            },
            ClosedOrder => SlackResponse {
                response_type: ResponseType::InChannel,
                text: format!("No longer taking orders"),
                ..Default::default()
            },
            Clear => SlackResponse {
                response_type: ResponseType::InChannel,
                text: format!("🙍 So that's how it's going to be!"),
                ..Default::default()
            },
            Associations { associations } => {
                let associations = associations
                    .into_iter()
                    .map(|x| format!("{} \u{2192} {}", &x.slack_name, &x.sharebill_account))
                    .collect::<Vec<_>>()
                    .join("\n    ");

                SlackResponse {
                    text: format!(
                        "I have the following mappings from slack names \
                         to sharebill accounts:\n    {}",
                        &associations
                    ),
                    ..Default::default()
                }
            }
            NewAssociation {
                user_name,
                sharebill_account,
            } => SlackResponse {
                text: format!(
                    "Billing orders by {} to account {}. Got it 👍",
                    user_name, sharebill_account
                ),
                ..Default::default()
            },
            Sharebill { url } => SlackResponse {
                response_type: ResponseType::InChannel,
                text: format!("💸 Posted to <{}|Sharebill> and closed order ✔️", url),
                ..Default::default()
            },
            Overhead { overhead_in_cents } => SlackResponse {
                text: format!(
                    "💁 Overhead is set to {}.{:02}",
                    overhead_in_cents / 100,
                    overhead_in_cents % 100
                ),
                ..Default::default()
            },
            OverheadSet {
                prev_overhead_in_cents,
                new_overhead_in_cents,
            } => SlackResponse {
                response_type: ResponseType::InChannel,
                text: format!(
                    "💁 Overhead changed from {}.{:02} to {}.{:02}",
                    prev_overhead_in_cents / 100,
                    prev_overhead_in_cents % 100,
                    new_overhead_in_cents / 100,
                    new_overhead_in_cents % 100
                ),
                ..Default::default()
            },
            Summary { orders } => {
                // writeln! cannot return Err when writing to a String. unwrap() below is Ok
                let mut buf = String::new();

                for (person_name, items) in orders {
                    writeln!(&mut buf, "{}:", person_name).unwrap();
                    for menu_item in items {
                        writeln!(&mut buf, " - {}. {}", menu_item.number, menu_item.name).unwrap();
                    }
                }

                SlackResponse {
                    text: buf,
                    ..Default::default()
                }
            }
            Price {
                overhead,
                overhead_per_person,
                summary,
            } => {
                // writeln! cannot return Err when writing to a String. unwrap() below is Ok
                let mut buf = String::new();

                if !overhead.is_zero() {
                    writeln!(
                        &mut buf,
                        "Total overhead {}, per person: {}",
                        overhead, overhead_per_person
                    )
                    .unwrap();
                }

                for (person_name, total, items) in summary {
                    writeln!(&mut buf, "{}: {:.2}", person_name, total).unwrap();
                    for menu_item in items {
                        writeln!(
                            &mut buf,
                            " - {}. {}: {:.2}",
                            menu_item.number,
                            menu_item.name,
                            menu_item.price_in_cents as f64 / 100.
                        )
                        .unwrap();
                    }
                }

                SlackResponse {
                    text: buf,
                    ..Default::default()
                }
            }
            Suggest { balances } => {
                let mut buf = String::new();

                writeln!(&mut buf, "💁 The poorest people on sharebill are:").unwrap();
                for (account_name, old_balance, new_balance) in balances {
                    writeln!(
                        &mut buf,
                        " - {} ({}, projected new balance: {})",
                        account_name,
                        old_balance.0.to_integer(),
                        new_balance.0.to_integer()
                    )
                    .unwrap();
                }

                SlackResponse {
                    response_type: ResponseType::InChannel,
                    text: buf,
                    ..Default::default()
                }
            }
            Help => SlackResponse {
                text: "USAGE: /ffs command args...\n\
                    associate [SLACK_NAME] SHAREBILL_ACCOUNT\n    Associate the given slack name (defaults to your name) with the given sharebill account\n\
                    associate\n    Display all slack name-sharebill account associations\n\
                    clear\n    Withdraw all your current orders\n\
                    closeorder\n    Close the current order\n\
                    help\n    This help\n\
                    openorder RESTAURANT\n    Start a new order from the given restaurant\n\
                    order QUERY\n    Order whatever matches QUERY in the menu\n\
                    overhead [VALUE]\n    Get/set overhead (delivery cost, gratuity, etc) for current order\n\
                    price\n    Like summary, but with price annotations\n\
                    repeat\n    Repeat your last order for the current restaurant\n\
                    restaurants\n    List known restaurants\n\
                    search QUERY\n    See what matches QUERY in the menu\n\
                    sharebill [CREDIT_ACCOUNT]\n    Post order to Sharebill. CREDIT_ACCOUNT defaults to your account\n\
                    sudo USER args...\n    Perform the command specified in args as USER\n\
                    suggest\n    Suggest who should pay for the order based on Sharebill balance\n\
                    summary\n    See the current order\n\
                    ".to_owned(),
                ..Default::default()
            },
        }
    }
}

fn slack_core(maybe_slack_token: &Option<&str>, req: &mut Request) -> Result<SlackResponse, Error> {
    let hashmap = req.get::<UrlEncodedBody>()?;

    if let &Some(slack_token) = maybe_slack_token {
        let given_token = hashmap
            .get("token")
            .and_then(|tokens| tokens.get(0))
            .map(String::as_ref);

        if given_token != Some(slack_token) {
            return Err(Error::InvalidSlackToken);
        }
    }

    if hashmap.contains_key("sslcheck") {
        return Ok(SlackResponse {
            text: String::new(),
            ..Default::default()
        });
    }

    let ref state_mutex = req.extensions.get::<web::StateContainer>().unwrap().0;
    let ref env = req.extensions.get::<web::EnvContainer>().unwrap().0;

    let text = &hashmap.get("text").ok_or(Error::MissingArgument("text"))?[0];
    let mut split = text.splitn(2, ' ');
    let cmd = split.next().unwrap();
    let args = split.next().unwrap_or("");

    let user_name = &hashmap
        .get("user_name")
        .ok_or(Error::MissingArgument("user_name"))?[0];

    exec_cmd(
        cmd,
        &CommandContext {
            state_mutex: &state_mutex,
            args: args,
            user_name: user_name,
            env: &env,
        },
    )
    .map(Into::into)
}

pub fn slack(slack_token: &Option<&str>, req: &mut Request) -> IronResult<Response> {
    match slack_core(slack_token, req) {
        Ok(response) => Ok(Response::with((
            status::Ok,
            serde_json::to_string(&response).unwrap(),
            Header(ContentType::json()),
        ))),
        Err(err) => Ok(Response::with((
            status::Ok,
            serde_json::to_string(&SlackResponse {
                text: format!("🙅 {:?}", &err),
                ..Default::default()
            })
            .unwrap(),
            Header(ContentType::json()),
        ))),
    }
}
