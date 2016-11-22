#[derive(Queryable, Serialize)]
pub struct Restaurant {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Serialize)]
pub struct MenuItem {
    pub restaurant: i32,
    pub id: i32,
    pub name: String,
    pub price_in_cents: i32,
}
