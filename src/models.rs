#[derive(Queryable, Serialize)]
pub struct Resturant {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Serialize)]
pub struct MenuItem {
    pub resturant: i32,
    pub id: i32,
    pub name: String,
    pub price_in_cents: i32,
}
