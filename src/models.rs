use std;
use diesel;
use diesel::types::*;
use schema::{menu_items, order_items};

#[derive(Copy, Clone, Debug, Serialize)]
pub struct RestaurantId(i32);

impl FromSql<Integer, diesel::sqlite::Sqlite> for RestaurantId {
    fn from_sql(bytes: Option<&<diesel::sqlite::Sqlite as diesel::backend::Backend>::RawValue>) -> Result<Self, Box<std::error::Error + Send + Sync>> {
        FromSql::<Integer, diesel::sqlite::Sqlite>::from_sql(bytes)
            .map(|x| RestaurantId(x))
    }
}

impl FromSqlRow<Integer, diesel::sqlite::Sqlite> for RestaurantId {
    fn build_from_row<T>(row: &mut T) -> Result<Self, Box<std::error::Error + Send + Sync>>
        where T : diesel::row::Row<diesel::sqlite::Sqlite>
    {
        FromSqlRow::<Integer, diesel::sqlite::Sqlite>::build_from_row(row)
            .map(|x| RestaurantId(x))
    }
}

impl From<i32> for RestaurantId {
    fn from(src: i32) -> Self { RestaurantId(src) }
}

impl From<RestaurantId> for i32 {
    fn from(src: RestaurantId) -> Self { src.0 }
}

#[derive(Debug, Queryable, Serialize)]
pub struct Restaurant {
    pub id: RestaurantId,
    pub name: String,
}

#[derive(Debug, Queryable, Serialize)]
pub struct Menu {
    pub id: i32,
    pub restaurant: RestaurantId,
    pub imported: i32,
}

#[derive(Debug, Queryable, Serialize, Identifiable, Associations)]
#[has_many(order_items, foreign_key="menu_item")]
pub struct MenuItem {
    pub id: i32,
    pub menu: i32,
    pub number: i32,
    pub name: String,
    pub price_in_cents: i32,
}

#[derive(Debug, Queryable, Serialize)]
pub struct Order {
    pub id: i32,
    pub menu: i32,
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

#[derive(Debug, Queryable, Serialize)]
pub struct SharebillAssociation {
    pub slack_name: String,
    pub sharebill_account: String,
}
