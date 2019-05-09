use models::*;
use words::*;

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

pub enum Response {
    RepeatNoMatch,
    OrderNoMatch {
        search_string: String,
    },
    PlacedOrder {
        menu_items: Vec<MenuItem>,
    },
    Restaurants {
        restaurants: Vec<Restaurant>,
    },
    RestaurantsNoMatch {
        restaurants: Vec<Restaurant>,
    },
    OpenedOrder {
        menu_url: String,
        restaurant_name: String,
    },
    ClosedOrder,
    Clear,
    Associations {
        associations: Vec<SharebillAssociation>,
    },
    NewAssociation {
        user_name: String,
        sharebill_account: String,
    },
    Sharebill {
        url: String,
    },
    Overhead {
        overhead_in_cents: i32,
    },
    OverheadSet {
        prev_overhead_in_cents: i32,
        new_overhead_in_cents: i32,
    },
    Summary {
        orders: Vec<(String, Vec<MenuItem>)>,
    },
}

impl Into<SlackResponse> for Response {
    fn into(self) -> SlackResponse {
        use self::Response::*;
        match self {
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
                use std::fmt::Write;
                let mut buf = String::new();

                // writeln! cannot return Err when writing to a String. unwrap() below is Ok

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
        }
    }
}
