use models::*;
use sharebill::Rational;

pub enum Response {
    UnknownCommand {
        cmd: String,
        args: String,
    },
    RepeatNoMatch,
    OrderNoMatch {
        search_string: String,
    },
    PlacedOrder {
        menu_items: Vec<MenuItem>,
    },
    SearchResults {
        query: String,
        items: Vec<MenuItem>,
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
    Price {
        overhead: Rational,
        overhead_per_person: Rational,
        summary: Vec<(String, f64, Vec<MenuItem>)>,
    },
    Suggest {
        balances: Vec<(String, Rational, Rational)>,
    },
    Help,
}
