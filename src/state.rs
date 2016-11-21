use diesel;
use models;

use diesel::prelude::*;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Diesel(err: diesel::result::Error) { from() }
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

    pub fn resturants(&self) -> Result<Vec<models::Resturant>, Error> {
        use schema::resturants::dsl::*;

        Ok(resturants.load::<models::Resturant>(&self.db_connection)?)
    }

    pub fn menu(&self, resturant_id: i32) -> Result<Vec<models::MenuItem>, Error> {
        use schema::menu_items::dsl::*;

        Ok(menu_items
            .filter(resturant.eq(resturant_id))
            .load::<models::MenuItem>(&self.db_connection)?
        )
    }
}
