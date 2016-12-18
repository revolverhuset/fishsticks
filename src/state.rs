extern crate time;

use diesel;
use ingest;
use models;
use std;
use takedown;

use diesel::prelude::*;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Diesel(err: diesel::result::Error) { from() }
        Ingest(err: ingest::Error) { from() }
        OrderAlreadyOpen(current_open_order: models::Order) { }
        OrderAlreadyClosed(order: models::Order) { }
        CouldntCreateTransaction(err: diesel::result::Error) { }
        NoOpenOrder
        NotFound
    }
}

impl<T> std::convert::From<diesel::result::TransactionError<T>> for Error
    where Error: std::convert::From<T>
{
    fn from(err: diesel::result::TransactionError<T>) -> Error {
        match err {
            diesel::result::TransactionError::CouldntCreateTransaction(err) => {
                Error::CouldntCreateTransaction(err)
            }
            diesel::result::TransactionError::UserReturnedError(err) => {
                err.into()
            }
        }
    }
}

#[derive(Debug)]
pub enum Query<'a> {
    ExactInteger(i32),
    FuzzyString(&'a str),
}

impl<'a, 'b> Query<'a> where 'b: 'a {
    pub fn interpret_string(input: &'b str) -> Query<'a> {
        match input.parse::<i32>() {
            Ok(integer) => Query::ExactInteger(integer),
            Err(_) => Query::FuzzyString(input),
        }
    }
}

pub struct State {
    db_connection: diesel::sqlite::SqliteConnection,
}

fn timestamp() -> i32 {
    time::now().to_timespec().sec as i32
}

impl State {
    pub fn new(db_connection: diesel::sqlite::SqliteConnection) -> State {
        State {
            db_connection: db_connection,
        }
    }

    pub fn create_restaurant(&self, name: &str) -> Result<models::RestaurantId, Error> {
        use schema::restaurants;

        #[derive(Insertable)]
        #[table_name="restaurants"]
        struct NewRestaurant<'a> {
            name: &'a str
        }

        let new_restaurant = NewRestaurant { name: name };

        diesel::insert(&new_restaurant)
            .into(restaurants::table)
            .execute(&self.db_connection)?;

        let restaurant_id = restaurants::table
            .filter(restaurants::name.eq(name))
            .load::<models::Restaurant>(&self.db_connection)?
            [0].id;

        Ok(restaurant_id)
    }

    pub fn restaurants(&self) -> Result<Vec<models::Restaurant>, Error> {
        use schema::restaurants::dsl::*;

        Ok(restaurants.load::<models::Restaurant>(&self.db_connection)?)
    }

    pub fn restaurant(&self, restaurant_id: models::RestaurantId) -> Result<Option<models::Restaurant>, Error> {
        use schema::restaurants::dsl::*;

        Ok(restaurants
            .find(i32::from(restaurant_id))
            .load::<models::Restaurant>(&self.db_connection)?
            .pop())
    }

    pub fn restaurant_by_name(&self, query_name: &str) -> Result<Option<models::Restaurant>, Error> {
        use schema::restaurants::dsl::*;

        Ok(restaurants
            .filter(name.eq(query_name))
            .limit(1)
            .load::<models::Restaurant>(&self.db_connection)?
            .pop())
    }

    pub fn menus_for_restaurant(&self, restaurant_id: models::RestaurantId) -> Result<Vec<models::Menu>, Error> {
        use schema::menus::dsl::*;

        Ok(menus
            .filter(restaurant.eq(i32::from(restaurant_id)))
            .order(imported.desc())
            .load::<models::Menu>(&self.db_connection)?
        )
    }

    pub fn current_menu_for_restaurant(&self, restaurant_id: models::RestaurantId) -> Result<models::Menu, Error> {
        use schema::menus::dsl::*;

        Ok(menus
            .filter(restaurant.eq(i32::from(restaurant_id)))
            .order(imported.desc())
            .limit(1)
            .load::<models::Menu>(&self.db_connection)?
            .pop().ok_or(Error::NotFound)?
        )
    }

    pub fn menu_object(&self, menu_id: i32) -> Result<Option<models::Menu>, Error> {
        use schema::menus::dsl::*;

        Ok(menus
            .find(menu_id)
            .load::<models::Menu>(&self.db_connection)?
            .pop()
        )
    }

    pub fn menu(&self, menu_id: i32) -> Result<Vec<models::MenuItem>, Error> {
        use schema::menu_items::dsl::*;

        Ok(menu_items
            .filter(menu.eq(menu_id))
            .load::<models::MenuItem>(&self.db_connection)?
        )
    }

    pub fn ingest_menu(&self, restaurant_id: models::RestaurantId, menu: &takedown::Menu) -> Result<(), Error> {
        self.db_connection.transaction(|| {
            ingest::menu(&self.db_connection, i32::from(restaurant_id), menu)
        })?;
        Ok(())
    }

    pub fn current_open_order(&self) -> Result<Option<models::Order>, Error> {
        use schema::orders::dsl::*;

        Ok(orders
            .filter(closed.is_null())
            .limit(1)
            .load::<models::Order>(&self.db_connection)?
            .pop())
    }

    pub fn demand_open_order(&self) -> Result<models::Order, Error> {
        self.current_open_order()?.ok_or(Error::NoOpenOrder)
    }

    pub fn create_order(&self, menu_id: i32) -> Result<(), Error> {
        use schema::orders;

        #[derive(Insertable)]
        #[table_name="orders"]
        struct NewOrder {
            pub menu: i32,
            pub overhead_in_cents: i32,
            pub opened: i32,
        }

        self.db_connection.transaction(|| {
            if let Some(current) = self.current_open_order()? {
                return Err(Error::OrderAlreadyOpen(current));
            }

            let new_order = NewOrder {
                menu: menu_id,
                overhead_in_cents: 0,
                opened: timestamp(),
            };

            diesel::insert(&new_order).into(orders::table)
                .execute(&self.db_connection)?;

            Ok(())
        })?;

        Ok(())
    }

    pub fn close_current_order(&self) -> Result<(), Error> {
        use schema::orders::dsl::*;

        self.db_connection.transaction(|| {
            let current = self.demand_open_order()?;

            if current.closed.is_some() {
                return Err(Error::OrderAlreadyClosed(current));
            }

            diesel::update(orders.find(current.id))
                .set(closed.eq(timestamp()))
                .execute(&self.db_connection)?;

            Ok(())
        })?;
        Ok(())
    }

    pub fn query_menu(&self, menu_id: i32, query: &Query) -> Result<Option<models::MenuItem>, Error> {
        use schema::menu_items::dsl::*;

        Ok(match *query {
            Query::ExactInteger(integer) =>
                menu_items.filter(number.eq(integer)).into_boxed(),
            Query::FuzzyString(string) =>
                menu_items
                    .filter(name.eq(string))
                    .into_boxed(),
        }
            .filter(menu.eq(menu_id))
            .limit(1)
            .load::<models::MenuItem>(&self.db_connection)?
            .pop())
    }

    pub fn add_order_item(&self, order: i32, person_name: &str, menu_item: i32) -> Result<(), Error> {
        use schema::order_items;

        #[derive(Insertable)]
        #[table_name="order_items"]
        struct NewOrderItem<'a> {
            pub order: i32,
            pub person_name: &'a str,
            pub menu_item: i32,
        }

        let new_order_item = NewOrderItem {
            order: order,
            person_name: person_name,
            menu_item: menu_item,
        };

        diesel::insert(&new_order_item).into(order_items::table)
            .execute(&self.db_connection)?;

        Ok(())
    }

    pub fn items_in_order(&self, order_id: i32) -> Result<Vec<(models::MenuItem, models::OrderItem)>, Error> {
        use schema::order_items::dsl::*;
        use schema::menu_items;

        let oitems = order_items
            .filter(order.eq(order_id))
            .order(person_name.asc())
            .load::<models::OrderItem>(&self.db_connection)?;

        let mut result = Vec::<(models::MenuItem, models::OrderItem)>::new();

        // Join manually, because I am unable to get Diesel to do it for me :(
        for oitem in oitems {
            result.push((
                menu_items::table
                    .find(oitem.menu_item)
                    .load(&self.db_connection)?
                    .pop().unwrap(),
                oitem,
            ));
        }

        Ok(result)
    }

    pub fn set_association(&self, slack_name: &str, sharebill_account: &str) -> Result<(), Error> {
        use schema::sharebill_associations;

        #[derive(Insertable)]
        #[table_name="sharebill_associations"]
        struct NewItem<'a> {
            slack_name: &'a str,
            sharebill_account: &'a str,
        }

        let new_item = NewItem {
            slack_name: slack_name,
            sharebill_account: sharebill_account,
        };

        diesel::insert_or_replace(&new_item)
            .into(sharebill_associations::table)
            .execute(&self.db_connection)?;

        Ok(())
    }

    pub fn all_associations(&self) -> Result<Vec<models::SharebillAssociation>, Error> {
        use schema::sharebill_associations::dsl::*;

        Ok(sharebill_associations
            .order(slack_name.asc())
            .load::<models::SharebillAssociation>(&self.db_connection)?
        )
    }
}
