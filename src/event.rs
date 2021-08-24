//! Struct and Trait correlated to Event
use crate::error::ApplyError;
use crate::prelude::EventEnvelope;
use crate::{message::EventMetadatas, Application, SubscriberManager};
use actix::{Actor, Addr, Recipient};
use event_store::prelude::RecordedEvent;
use futures::Future;

pub(crate) mod handler;

pub use handler::EventHandler;
pub use handler::EventHandlerInstance;

pub type BoxedResolver<A> =
    Box<dyn Fn(RecordedEvent, actix::Addr<SubscriberManager<A>>) -> std::result::Result<(), ()>>;

pub type BoxedDeserializer = Box<dyn Fn(RecordedEvent) -> Box<dyn ErasedGeneric>>;

/// Receive an immutable event to handle
pub trait Handler<E: event_store::Event> {
    fn handle(
        &mut self,
        event: &E,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), ()>> + Send>>;
}

/// Define an Event which can be produced and consumed
pub trait Event: event_store::prelude::Event {
    fn into_envelope<'de>(event: RecordedEvent) -> Result<crate::message::EventEnvelope<Self>, ()>
    where
        Self: serde::Deserialize<'de> + serde::de::Deserialize<'de>,
        Self: 'static + Clone,
    {
        let r = Self::deserialize(event.data).map_err(|_| ())?;

        Ok(crate::message::EventEnvelope {
            event: r,
            meta: EventMetadatas {
                correlation_id: event.correlation_id,
                causation_id: event.causation_id,
                stream_name: event.stream_uuid,
            },
        })
    }

    fn into_erased<'de>() -> BoxedDeserializer
    where
        Self: serde::Deserialize<'de> + serde::de::Deserialize<'de>,
        Self: 'static + Clone + GenericEvent,
    {
        Box::new(|event: RecordedEvent| -> Box<dyn ErasedGeneric> {
            Box::new(Self::deserialize(event.data).unwrap())
        })
    }

    fn lazy_deserialize<'de, A: Application>() -> BoxedResolver<A>
    where
        Self: serde::Deserialize<'de> + serde::de::Deserialize<'de>,
        Self: 'static + Clone,
    {
        Box::new(
            |event: RecordedEvent, resolver: actix::Addr<SubscriberManager<A>>| -> Result<(), ()> {
                resolver.do_send(Self::into_envelope(event)?);

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

pub trait Querializer {}

pub trait GenericEvent {
    // Not object safe because of this generic method.
    fn notify<Q: Querializer>(&self, querializer: Q);
}

impl<'a, T: ?Sized> Querializer for &'a T where T: Querializer {}

impl<'a, T: ?Sized> GenericEvent for Box<T>
where
    T: GenericEvent,
{
    fn notify<Q: Querializer>(&self, querializer: Q) {
        (**self).notify(querializer)
    }
}

/////////////////////////////////////////////////////////////////////
// This is an object-safe equivalent that interoperates seamlessly.

pub trait ErasedGeneric: 'static + Send {
    fn erased_fn(&self, querializer: &Querializer);
}

impl GenericEvent for ErasedGeneric {
    fn notify<Q: Querializer>(&self, querializer: Q) {
        self.erased_fn(&querializer)
    }
}

impl<T> ErasedGeneric for T
where
    T: GenericEvent + 'static + Send,
{
    fn erased_fn(&self, querializer: &Querializer) {
        self.notify(querializer)
    }
}

pub struct EventPlaceHolder;
impl Querializer for EventPlaceHolder {}

// impl Event for ErasedGeneric {}
