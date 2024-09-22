pub mod postgre;
mod storage;

pub use storage::Webpage;
pub use storage::Link;
pub use postgre::PostgresStorage;