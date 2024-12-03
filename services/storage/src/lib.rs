pub mod postgre;
mod schema;

pub use schema::Webpage;
pub use schema::Link;
pub use postgre::PostgresStorage;