pub mod appender;
pub mod error;

#[cfg(feature = "inmemory_storage")]
pub use event_store_storage_inmemory::InMemoryStorage;

#[cfg(feature = "postgres_storage")]
pub use event_store_storage_postgres::PostgresStorage;

pub mod reader;

#[cfg(test)]
mod test;
