pub mod postgre;
mod models;

pub use models::Webpage;
pub use models::Link;
pub use postgre::PostgresStorage;