extern crate strsim;
extern crate time;

use diesel;
use ingest;
use models::*;
use std;
use takedown;

use diesel::prelude::*;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Diesel(err: diesel::result::Error) { from() }
        Ingest(err: ingest::Error) { from() }
        OrderAlreadyOpen(current_open_order: Order) { }
        OrderAlreadyClosed(order: Order) { }
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

fn distance(a: &str, b: &str) -> usize {
    ((1.-strsim::jaro_winkler(&a.to_lowercase(), &b.to_lowercase())) * 1000.) as usize
}

impl State {
    pub fn new(db_connection: diesel::sqlite::SqliteConnection) -> State {
        State {
            db_connection: db_connection,
        }
    }

    pub fn create_restaurant(&self, name: &str) -> Result<RestaurantId, Error> {
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
            .load::<Restaurant>(&self.db_connection)?
            [0].id;

        Ok(restaurant_id)
    }

    pub fn restaurants(&self) -> Result<Vec<Restaurant>, Error> {
        use schema::restaurants::dsl::*;

        Ok(restaurants.load::<Restaurant>(&self.db_connection)?)
    }

    pub fn restaurant(&self, restaurant_id: RestaurantId) -> Result<Option<Restaurant>, Error> {
        use schema::restaurants::dsl::*;

        Ok(restaurants
            .find(i32::from(restaurant_id))
            .load::<Restaurant>(&self.db_connection)?
            .pop())
    }

    pub fn restaurant_by_name(&self, query_name: &str) -> Result<Option<Restaurant>, Error> {
        use schema::restaurants::dsl::*;

        Ok(restaurants
            .filter(name.eq(query_name))
            .limit(1)
            .load::<Restaurant>(&self.db_connection)?
            .pop())
    }

    pub fn menus_for_restaurant(&self, restaurant_id: RestaurantId) -> Result<Vec<Menu>, Error> {
        use schema::menus::dsl::*;

        Ok(menus
            .filter(restaurant.eq(i32::from(restaurant_id)))
            .order(imported.desc())
            .load::<Menu>(&self.db_connection)?
        )
    }

    pub fn current_menu_for_restaurant(&self, restaurant_id: RestaurantId) -> Result<Menu, Error> {
        use schema::menus::dsl::*;

        Ok(menus
            .filter(restaurant.eq(i32::from(restaurant_id)))
            .order(imported.desc())
            .limit(1)
            .load::<Menu>(&self.db_connection)?
            .pop().ok_or(Error::NotFound)?
        )
    }

    pub fn menu_object(&self, menu_id: MenuId) -> Result<Option<Menu>, Error> {
        use schema::menus::dsl::*;

        Ok(menus
            .find(i32::from(menu_id))
            .load::<Menu>(&self.db_connection)?
            .pop()
        )
    }

    pub fn menu(&self, menu_id: MenuId) -> Result<Vec<MenuItem>, Error> {
        use schema::menu_items::dsl::*;

        Ok(menu_items
            .filter(menu.eq(i32::from(menu_id)))
            .load::<MenuItem>(&self.db_connection)?
        )
    }

    pub fn ingest_menu(&self, restaurant_id: RestaurantId, menu: &takedown::Menu) -> Result<(), Error> {
        self.db_connection.transaction(|| {
            ingest::menu(&self.db_connection, i32::from(restaurant_id), menu)
        })?;
        Ok(())
    }

    pub fn current_open_order(&self) -> Result<Option<Order>, Error> {
        use schema::orders::dsl::*;

        Ok(orders
            .filter(closed.is_null())
            .limit(1)
            .load::<Order>(&self.db_connection)?
            .pop())
    }

    pub fn demand_open_order(&self) -> Result<Order, Error> {
        self.current_open_order()?.ok_or(Error::NoOpenOrder)
    }

    pub fn create_order(&self, menu_id: MenuId) -> Result<(), Error> {
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
                menu: i32::from(menu_id),
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

            diesel::update(orders.find(i32::from(current.id)))
                .set(closed.eq(timestamp()))
                .execute(&self.db_connection)?;

            Ok(())
        })?;
        Ok(())
    }

    pub fn set_overhead(&self, order_id: OrderId, new_overhead_in_cents: i32) -> Result<(), Error> {
        use schema::orders::dsl::*;

        diesel::update(orders.find(i32::from(order_id)))
            .set(overhead_in_cents.eq(new_overhead_in_cents))
            .execute(&self.db_connection)?;

        Ok(())
    }

    pub fn query_menu(&self, menu_id: MenuId, query: &Query) -> Result<Vec<MenuItem>, Error> {
        use schema::menu_items::dsl::*;

        let all_items = menu_items
            .filter(menu.eq(i32::from(menu_id)));

        match *query {
            Query::ExactInteger(integer) =>
                Ok(all_items
                    .filter(number.eq(integer))
                    .limit(1)
                    .load::<MenuItem>(&self.db_connection)?
                ),
            Query::FuzzyString(string) => {
                let mut items = all_items.load::<MenuItem>(&self.db_connection)?;
                items.sort_by_key(|x| distance(&string, &x.name));
                Ok(items)
            }
        }
    }

    pub fn add_order_item(&self, order: OrderId, person_name: &str, menu_item: MenuItemId) -> Result<(), Error> {
        use schema::order_items;

        #[derive(Insertable)]
        #[table_name="order_items"]
        struct NewOrderItem<'a> {
            pub order: i32,
            pub person_name: &'a str,
            pub menu_item: i32,
        }

        let new_order_item = NewOrderItem {
            order: i32::from(order),
            person_name: person_name,
            menu_item: i32::from(menu_item),
        };

        diesel::insert(&new_order_item).into(order_items::table)
            .execute(&self.db_connection)?;

        Ok(())
    }

    pub fn clear_orders_for_person(&self, order: OrderId, person_name: &str) -> Result<(), Error> {
        use schema::order_items;

        diesel::delete(
            order_items::table
                .filter(order_items::order.eq(i32::from(order)))
                .filter(order_items::person_name.eq(person_name))
        )
            .execute(&self.db_connection)?;

        Ok(())
    }

    pub fn items_in_order(&self, order_id: OrderId) -> Result<Vec<(MenuItem, OrderItem)>, Error> {
        use schema::order_items::dsl::*;
        use schema::menu_items;

        let oitems = order_items
            .filter(order.eq(i32::from(order_id)))
            .order((person_name.asc(), menu_item.asc()))
            .load::<OrderItem>(&self.db_connection)?;

        let mut result = Vec::<(MenuItem, OrderItem)>::new();

        // Join manually, because I am unable to get Diesel to do it for me :(
        for oitem in oitems {
            result.push((
                menu_items::table
                    .find(i32::from(oitem.menu_item))
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

    pub fn all_associations(&self) -> Result<Vec<SharebillAssociation>, Error> {
        use schema::sharebill_associations::dsl::*;

        Ok(sharebill_associations
            .order(slack_name.asc())
            .load::<SharebillAssociation>(&self.db_connection)?
        )
    }
}
