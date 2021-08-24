//! Struct and Trait correlated to Event
use crate::error::ApplyError;
use crate::{message::EventMetadatas, Application, SubscriberManager};
use event_store::prelude::RecordedEvent;
use futures::Future;

pub(crate) mod handler;

pub use handler::EventHandler;
pub use handler::EventHandlerInstance;

pub type BoxedResolver<A> =
    Box<dyn Fn(RecordedEvent, actix::Addr<SubscriberManager<A>>) -> std::result::Result<(), ()>>;

/// Receive an immutable event to handle
pub trait Handler<E: event_store::Event> {
    fn handle(
        &mut self,
        event: &E,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), ()>> + Send>>;
}

/// Define an Event which can be produced and consumed
pub trait Event: event_store::prelude::Event {
    fn lazy_deserialize<'de, A: Application>() -> BoxedResolver<A>
    where
        Self: serde::Deserialize<'de> + serde::de::Deserialize<'de>,
        Self: 'static + Clone,
    {
        Box::new(
            |event: RecordedEvent, resolver: actix::Addr<SubscriberManager<A>>| -> Result<(), ()> {
                let r = Self::deserialize(event.data).map_err(|_| ())?;

                resolver.do_send(crate::message::EventEnvelope {
                    event: r,
                    meta: EventMetadatas {
                        correlation_id: event.correlation_id,
                        causation_id: event.causation_id,
                        stream_name: event.stream_uuid,
                    },
                });

                Ok(())
            },
        )
    }

    fn register<'de, A: Application>() -> (Vec<&'static str>, BoxedResolver<A>)
    where
        Self: serde::Deserialize<'de> + serde::de::Deserialize<'de>,
        Self: 'static + Clone,
    {
        (Self::all_event_types(), Self::lazy_deserialize())
    }
}

/// Define an event applier
pub trait EventApplier<E: Event> {
    fn apply(&mut self, event: &E) -> Result<(), ApplyError>;
}
