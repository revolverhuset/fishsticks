use schema::{menu_items, order_items};

#[derive(Debug, Queryable, Serialize)]
pub struct Restaurant {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Queryable, Serialize, Identifiable, Associations)]
#[has_many(order_items, foreign_key="menu_item")]
pub struct MenuItem {
    pub id: i32,
    pub restaurant: i32,
    pub number: i32,
    pub name: String,
    pub price_in_cents: i32,
}

#[derive(Debug, Queryable, Serialize)]
pub struct Order {
    pub id: i32,
    pub restaurant: i32,
    pub overhead_in_cents: i32,
    pub opened: i32,
    pub closed: Option<i32>,
}

#[derive(Debug, Queryable, Serialize, Identifiable, Associations)]
#[belongs_to(MenuItem)]
pub struct OrderItem {
    pub id: i32,
    pub order: i32,
    pub person_name: String,
    pub menu_item: i32,
}
