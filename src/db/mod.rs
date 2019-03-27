use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;

pub mod models;
pub mod schema;
pub mod types;

pub fn connect() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect("Error connecting to postgres")
}
