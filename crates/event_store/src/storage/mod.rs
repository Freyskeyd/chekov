pub mod appender;

#[cfg(feature = "inmemory")]
mod inmemory;
#[cfg(feature = "inmemory")]
pub use inmemory::InMemoryStorage;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::PostgresStorage;

pub mod reader;

#[cfg(test)]
mod test;
