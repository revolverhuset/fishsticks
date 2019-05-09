use state;
use web;

use itertools::*;
use sharebill::Rational;
use std::collections::HashMap;

use super::command_context::CommandContext;
use super::error::*;
use super::response::*;

fn cmd_repeat(
    &CommandContext {
        state_mutex,
        user_name,
        ..
    }: &CommandContext,
) -> Result<Response, Error> {
    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;
    let menu = state
        .menu_object(open_order.menu)?
        .expect("Database invariant");

    let menu_items = state.previous_orders(user_name, menu.restaurant)?;

    let menu_items = menu_items
        .into_iter()
        .map(|menu_item| -> Result<_, Error> {
            let query = state::Query::ExactInteger(menu_item.number);
            Ok(state.query_menu(open_order.menu, &query)?.pop())
        })
        .collect::<Result<Vec<_>, Error>>()?
        .into_iter()
        .filter_map(|x| x)
        .collect::<Vec<_>>();

    if menu_items.is_empty() {
        return Ok(Response::RepeatNoMatch);
    }

    for menu_item in menu_items.iter() {
        state.add_order_item(open_order.id, user_name, menu_item.id)?;
    }

    Ok(Response::PlacedOrder { menu_items })
}

fn cmd_restaurants(
    &CommandContext { state_mutex, .. }: &CommandContext,
) -> Result<Response, Error> {
    let state = state_mutex.lock()?;
    let restaurants = state.restaurants()?;

    Ok(Response::Restaurants { restaurants })
}

fn cmd_openorder(
    &CommandContext {
        state_mutex,
        args,
        env: &web::Env { ref base_url, .. },
        ..
    }: &CommandContext,
) -> Result<Response, Error> {
    let state = state_mutex.lock()?;

    let restaurant = match state.restaurant_by_name(args)? {
        Some(resturant) => resturant,
        None => {
            return Ok(Response::RestaurantsNoMatch {
                restaurants: state.restaurants()?,
            })
        }
    };

    let menu = state.current_menu_for_restaurant(restaurant.id)?;

    state.create_order(menu.id)?;

    let menu_url = format!("{}menu/{}", base_url, i32::from(menu.id));

    Ok(Response::OpenedOrder {
        menu_url,
        restaurant_name: restaurant.name,
    })
}

fn cmd_closeorder(&CommandContext { state_mutex, .. }: &CommandContext) -> Result<Response, Error> {
    let state = state_mutex.lock()?;

    state.close_current_order()?;

    Ok(Response::ClosedOrder)
}

fn cmd_search(
    &CommandContext {
        state_mutex, args, ..
    }: &CommandContext,
) -> Result<Response, Error> {
    let query = state::Query::interpret_string(&args);

    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;

    let items = state.query_menu(open_order.menu, &query)?;

    Ok(Response::SearchResults {
        query: args.to_string(),
        items,
    })
}

fn cmd_order(
    &CommandContext {
        state_mutex,
        args,
        user_name,
        ..
    }: &CommandContext,
) -> Result<Response, Error> {
    let query = state::Query::interpret_string(&args);

    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;

    match state.query_menu(open_order.menu, &query)?.pop() {
        Some(menu_item) => {
            state.add_order_item(open_order.id, user_name, menu_item.id)?;

            Ok(Response::PlacedOrder {
                menu_items: vec![menu_item],
            })
        }
        None => Ok(Response::OrderNoMatch {
            search_string: args.to_string(),
        }),
    }
}

fn cmd_clear(
    &CommandContext {
        state_mutex,
        user_name,
        ..
    }: &CommandContext,
) -> Result<Response, Error> {
    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;

    state.clear_orders_for_person(open_order.id, user_name)?;

    Ok(Response::Clear)
}

fn cmd_summary(&CommandContext { state_mutex, .. }: &CommandContext) -> Result<Response, Error> {
    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;
    let items = state.items_in_order(open_order.id)?;

    Ok(Response::Summary {
        orders: items
            .into_iter()
            .group_by(|&(_, ref order_item)| order_item.person_name.clone())
            .into_iter()
            .map(|(person_name, items)| {
                (
                    person_name,
                    items.map(|(menu_item, _order_item)| menu_item).collect(),
                )
            })
            .collect(),
    })
}

fn cmd_price(&CommandContext { state_mutex, .. }: &CommandContext) -> Result<Response, Error> {
    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;
    let items = state.items_in_order(open_order.id)?;

    let persons = Rational::from(
        items
            .iter()
            .group_by(|&&(_, ref order_item)| order_item.person_name.clone())
            .into_iter()
            .count(),
    );

    let overhead = Rational::from_cents(open_order.overhead_in_cents);
    let overhead_per_person = overhead.clone() / persons; // Divide by zero for empty orders

    let summary = items
        .into_iter()
        .group_by(|&(_, ref order_item)| order_item.person_name.clone())
        .into_iter()
        .map(|(person_name, items)| {
            let items = items.map(|(menu_item, _)| menu_item).collect::<Vec<_>>();
            let total = items
                .iter()
                .map(|ref menu_item| menu_item.price_in_cents)
                .sum();
            let total = Rational::from_cents(total) + &overhead_per_person;
            let total = total.to_f64();
            (person_name, total, items)
        })
        .collect::<Vec<_>>();

    Ok(Response::Price {
        overhead,
        overhead_per_person,
        summary,
    })
}

fn cmd_associate(
    &CommandContext {
        state_mutex,
        args,
        user_name,
        ..
    }: &CommandContext,
) -> Result<Response, Error> {
    if args.len() == 0 {
        let state = state_mutex.lock()?;
        let associations = state.all_associations()?;

        Ok(Response::Associations { associations })
    } else {
        let split = args.split_whitespace().collect::<Vec<_>>();
        let (slack_name, sharebill_account) = match split.len() {
            1 => (user_name, split[0]),
            2 => (split[0], split[1]),
            _ => return Err(Error::InputError),
        };

        let state = state_mutex.lock()?;
        state.set_association(slack_name, sharebill_account)?;

        Ok(Response::NewAssociation {
            user_name: slack_name.to_string(),
            sharebill_account: sharebill_account.to_string(),
        })
    }
}

fn generate_bill(state: &state::State) -> Result<HashMap<String, Rational>, Error> {
    use num::Zero;

    let open_order = state.demand_open_order()?;
    let items = state.items_in_order(open_order.id)?;

    let associations = state
        .all_associations()?
        .into_iter()
        .map(|x| (x.slack_name, x.sharebill_account))
        .collect::<HashMap<_, _>>();

    let persons = Rational::from(
        items
            .iter()
            .group_by(|&&(_, ref order_item)| order_item.person_name.clone())
            .into_iter()
            .count(),
    );

    let overhead = Rational::from_cents(open_order.overhead_in_cents);
    let overhead_per_person = overhead / persons;

    let slack_debits = items
        .into_iter()
        .group_by(|&(_, ref order_item)| order_item.person_name.clone())
        .into_iter()
        .map(|(person_name, items)| {
            let food = items
                .into_iter()
                .map(|(menu_item, _)| Rational::from_cents(menu_item.price_in_cents))
                .fold(Rational::zero(), |acc, x| acc + x);

            (person_name, food + &overhead_per_person)
        })
        .collect::<Vec<_>>();

    // Associations are deliberately used to bill orders by different
    // people to the same accuont. This is handled below:
    let mut debits = HashMap::<String, Rational>::new();
    for (slack_name, value) in slack_debits {
        let account = associations
            .get(&slack_name)
            .ok_or(Error::MissingAssociation(slack_name))?;
        let entry = debits.entry(account.clone()).or_insert_with(Rational::zero);
        *entry = &*entry + value;
    }

    Ok(debits)
}

fn cmd_sharebill(
    &CommandContext {
        state_mutex,
        args,
        user_name,
        env:
            &web::Env {
                ref maybe_sharebill_url,
                ref sharebill_cookies,
                ..
            },
        ..
    }: &CommandContext,
) -> Result<Response, Error> {
    use num::Zero;
    use std::collections::HashMap;

    let sharebill_url = maybe_sharebill_url
        .as_ref()
        .ok_or(Error::MissingConfig("web.sharebill_url"))?;

    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;

    let description = format!(
        "{}",
        state
            .restaurant(
                state
                    .menu_object(open_order.menu)?
                    .ok_or(Error::NotFound)?
                    .restaurant
            )?
            .ok_or(Error::NotFound)?
            .name
    );

    let associations = state
        .all_associations()?
        .into_iter()
        .map(|x| (x.slack_name, x.sharebill_account))
        .collect::<HashMap<_, _>>();

    let debits = generate_bill(&state)?;

    let credit_account = match args.len() {
        0 => associations.get(user_name).map(|x| x as &str),
        _ => Some(args),
    }
    .ok_or(Error::MissingAssociation(user_name.to_owned()))?;

    let total = debits.values().fold(Rational::zero(), |acc, x| acc + x);

    let mut credits = HashMap::<String, Rational>::new();
    credits.insert(credit_account.to_owned(), total);

    let post = sharebill::models::Post {
        meta: sharebill::models::Meta {
            description: description,
            timestamp: time::now_utc(),
        },
        transaction: sharebill::models::Transaction {
            debits: debits,
            credits: credits,
        },
    };

    let target_url = format!("{}post/{}", &sharebill_url, &uuid::Uuid::new_v4());

    let res = reqwest::Client::new()
        .request(reqwest::Method::PUT, &target_url)
        .header(reqwest::header::COOKIE, sharebill_cookies.join(", "))
        .json(&post)
        .send()?;

    if res.status() != reqwest::StatusCode::CREATED {
        return Err(Error::UnexpectedStatus(res.status().clone()));
    }

    state.close_current_order()?;

    Ok(Response::Sharebill { url: target_url })
}

fn cmd_suggest(
    &CommandContext {
        state_mutex,
        env:
            &web::Env {
                ref maybe_sharebill_url,
                ref sharebill_cookies,
                ..
            },
        ..
    }: &CommandContext,
) -> Result<Response, Error> {
    #[derive(Deserialize, Debug)]
    struct Row {
        pub key: String,
        pub value: Rational,
    }

    #[derive(Deserialize)]
    struct Balances {
        pub rows: Vec<Row>,
    }

    let sharebill_url = maybe_sharebill_url
        .as_ref()
        .ok_or(Error::MissingConfig("web.sharebill_url"))?;

    let state = state_mutex.lock()?;
    let debits = generate_bill(&state)?;

    let mut res = reqwest::Client::new()
        .request(reqwest::Method::GET, &format!("{}balances", &sharebill_url))
        .header(reqwest::header::COOKIE, sharebill_cookies.join(", "))
        .send()?;

    if !res.status().is_success() {
        return Err(Error::UnexpectedStatus(res.status().clone()));
    }
    let balances: Balances = res.json()?;

    let mut balances = balances
        .rows
        .into_iter()
        .filter(|row| debits.contains_key(&row.key))
        .map(|row| {
            let this_meal = debits
                .get(&row.key)
                .expect("Guaranteed by filter on the line above");
            let new_balance = &row.value - this_meal;
            (row.key, row.value, new_balance)
        })
        .collect::<Vec<_>>();

    balances.sort_by(|a, b| a.2.cmp(&b.2));
    let balances = balances.into_iter().take(3).collect();

    Ok(Response::Suggest { balances })
}

fn cmd_overhead(
    &CommandContext {
        state_mutex, args, ..
    }: &CommandContext,
) -> Result<Response, Error> {
    let state = state_mutex.lock()?;
    let open_order = state.demand_open_order()?;

    if args.len() == 0 {
        Ok(Response::Overhead {
            overhead_in_cents: open_order.overhead_in_cents,
        })
    } else {
        let prev_overhead_in_cents = open_order.overhead_in_cents;

        let overhead = args.parse::<f64>()?;
        let new_overhead_in_cents = (overhead * 100.0).round() as i32;

        state.set_overhead(open_order.id, new_overhead_in_cents)?;

        Ok(Response::OverheadSet {
            prev_overhead_in_cents,
            new_overhead_in_cents,
        })
    }
}

fn cmd_sudo(cmd_ctx: &CommandContext) -> Result<Response, Error> {
    let mut split = cmd_ctx.args.splitn(3, ' ');
    let user_name = split.next().unwrap();
    let cmd = split.next().ok_or(Error::MissingArgument("command"))?;
    let args = split.next().unwrap_or("");

    super::exec_cmd(
        cmd,
        &CommandContext {
            user_name: user_name,
            args: args,
            ..*cmd_ctx
        },
    )
}

fn cmd_help(_cmd_ctx: &CommandContext) -> Result<Response, Error> {
    Ok(Response::Help)
}

type CommandHandler = Fn(&CommandContext) -> Result<Response, Error> + Sync;

lazy_static! {
    pub static ref COMMAND_MAP: HashMap<&'static str, &'static CommandHandler> = {
        let mut m: HashMap<&'static str, &'static CommandHandler> = HashMap::new();
        m.insert("associate", &cmd_associate);
        m.insert("clear", &cmd_clear);
        m.insert("cancel", &cmd_clear);
        m.insert("reset", &cmd_clear);
        m.insert("closeorder", &cmd_closeorder);
        m.insert("help", &cmd_help);
        m.insert("openorder", &cmd_openorder);
        m.insert("open", &cmd_openorder);
        m.insert("order", &cmd_order);
        m.insert("overhead", &cmd_overhead);
        m.insert("tips", &cmd_overhead);
        m.insert("price", &cmd_price);
        m.insert("repeat", &cmd_repeat);
        m.insert("reorder", &cmd_repeat);
        m.insert("retweet", &cmd_repeat);
        m.insert("restaurants", &cmd_restaurants);
        m.insert("search", &cmd_search);
        m.insert("sharebill", &cmd_sharebill);
        m.insert("sudo", &cmd_sudo);
        m.insert("suggest", &cmd_suggest);
        m.insert("summary", &cmd_summary);
        m
    };
}
