use std;
use diesel;
use diesel::types::*;
use schema::{menu_items, order_items};

macro_rules! generate_id_type {
    ( $x:ident ) => {
        #[derive(Copy, Clone, Debug, Serialize)]
        pub struct $x(i32);

        impl FromSql<Integer, diesel::sqlite::Sqlite> for $x {
            fn from_sql(bytes: Option<&<diesel::sqlite::Sqlite as diesel::backend::Backend>::RawValue>) -> Result<Self, Box<std::error::Error + Send + Sync>> {
                FromSql::<Integer, diesel::sqlite::Sqlite>::from_sql(bytes)
                    .map(|x| $x(x))
            }
        }

        impl FromSqlRow<Integer, diesel::sqlite::Sqlite> for $x {
            fn build_from_row<T>(row: &mut T) -> Result<Self, Box<std::error::Error + Send + Sync>>
                where T : diesel::row::Row<diesel::sqlite::Sqlite>
            {
                FromSqlRow::<Integer, diesel::sqlite::Sqlite>::build_from_row(row)
                    .map(|x| $x(x))
            }
        }

        impl From<i32> for $x {
            fn from(src: i32) -> Self { $x(src) }
        }

        impl From<$x> for i32 {
            fn from(src: $x) -> Self { src.0 }
        }
    };
}

generate_id_type!(RestaurantId);
generate_id_type!(MenuId);

#[derive(Debug, Queryable, Serialize)]
pub struct Restaurant {
    pub id: RestaurantId,
    pub name: String,
}

#[derive(Debug, Queryable, Serialize)]
pub struct Menu {
    pub id: MenuId,
    pub restaurant: RestaurantId,
    pub imported: i32,
}

#[derive(Debug, Queryable, Serialize, Identifiable, Associations)]
#[has_many(order_items, foreign_key="menu_item")]
pub struct MenuItem {
    pub id: i32,
    pub menu: MenuId,
    pub number: i32,
    pub name: String,
    pub price_in_cents: i32,
}

#[derive(Debug, Queryable, Serialize)]
pub struct Order {
    pub id: i32,
    pub menu: MenuId,
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
