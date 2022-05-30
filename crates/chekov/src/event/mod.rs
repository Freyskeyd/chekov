//! Struct and Trait correlated to Event
use crate::error::{ApplyError, HandleError};
use event_store::prelude::RecordedEvent;
use futures::future::BoxFuture;
use std::any::TypeId;
use std::collections::BTreeMap;

pub(crate) mod handler;

#[doc(hidden)]
pub mod resolver;

pub use handler::EventHandler;
#[doc(hidden)]
pub use handler::EventHandlerInstance;

/// Define an Event which can be produced and consumed
// pub trait Event: event_store::prelude::Event {
pub trait Event: Send + Clone {}

/// Define an event applier
pub trait EventApplier<E: Event> {
    fn apply(&mut self, event: &E) -> Result<(), ApplyError>;
}

/// Receive an immutable event to handle
pub trait Handler<E: crate::event::Event> {
    fn handle(&mut self, event: &E) -> BoxFuture<Result<(), HandleError>>;
}
