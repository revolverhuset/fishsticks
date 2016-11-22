use diesel;
use models;
use ingest;
use takedown;

use diesel::prelude::*;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Diesel(err: diesel::result::Error) { from() }
        IngestError(err: diesel::result::TransactionError<ingest::Error>) { from() }
    }
}

pub struct State {
    db_connection: diesel::sqlite::SqliteConnection,
}

impl State {
    pub fn new(db_connection: diesel::sqlite::SqliteConnection) -> State {
        State {
            db_connection: db_connection,
        }
    }

    pub fn restaurants(&self) -> Result<Vec<models::Restaurant>, Error> {
        use schema::restaurants::dsl::*;

        Ok(restaurants.load::<models::Restaurant>(&self.db_connection)?)
    }

    pub fn restaurant_by_name(&self, query_name: &str) -> Result<Option<models::Restaurant>, Error> {
        use schema::restaurants::dsl::*;

        Ok(restaurants
            .filter(name.eq(query_name))
            .limit(1)
            .load::<models::Restaurant>(&self.db_connection)?
            .pop())
    }

    pub fn menu(&self, restaurant_id: i32) -> Result<Vec<models::MenuItem>, Error> {
        use schema::menu_items::dsl::*;

        Ok(menu_items
            .filter(restaurant.eq(restaurant_id))
            .load::<models::MenuItem>(&self.db_connection)?
        )
    }

    pub fn ingest_menu(&self, restaurant: &str, menu: &takedown::Menu) -> Result<(), Error> {
        self.db_connection.transaction(|| {
            ingest::restaurant(&self.db_connection, restaurant, menu)
        })?;
        Ok(())
    }

    pub fn create_order(&self, _restaurant_id: i32) -> Result<(), Error> {
        Ok(()) //unimplemented!()
    }
}
