use diesel::{SqliteConnection};
use diesel::r2d2::{self,ConnectionManager};

pub mod schema;
pub mod actions;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
//pub type Connection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;
