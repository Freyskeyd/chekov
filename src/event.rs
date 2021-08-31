//! Struct and Trait correlated to Event
use crate::error::ApplyError;
use crate::Aggregate;
use crate::{message::EventMetadatas, Application, SubscriberManager};
use actix::Actor;
use event_store::prelude::RecordedEvent;
use futures::Future;
use std::any::TypeId;
use std::collections::BTreeMap;

pub(crate) mod handler;

pub use handler::EventHandler;
pub use handler::EventHandlerInstance;

pub type BoxedResolver<A> =
    Box<dyn Fn(RecordedEvent, actix::Addr<SubscriberManager<A>>) -> std::result::Result<(), ()>>;

// pub type BoxedDeserializer = Box<dyn Fn(RecordedEvent) -> Box<dyn ErasedGeneric>>;

/// Receive an immutable event to handle
pub trait Handler<E: crate::event::Event> {
    fn handle(
        &mut self,
        event: &E,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), ()>> + Send>>;
}

/// Define an Event which can be produced and consumed
// pub trait Event: event_store::prelude::Event {
pub trait Event: Send {
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
        Self: 'static + Clone + event_store::Event,
    {
        (Self::all_event_types(), Self::lazy_deserialize())
    }
}

/// Define an event applier
pub trait EventApplier<E: Event> {
    fn apply(&mut self, event: &E) -> Result<(), ApplyError>;
}

#[doc(hidden)]
pub type EventResolverFn<T> =
    fn(
        event_store::prelude::RecordedEvent,
        actix::Addr<T>,
    ) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<(), ()>> + Send>>;

#[doc(hidden)]
pub struct EventResolverRegistry<T: Actor, A: Aggregate> {
    pub names: BTreeMap<&'static str, TypeId>,
    pub resolvers: BTreeMap<TypeId, EventResolverFn<T>>,
    pub appliers:
        BTreeMap<TypeId, fn(&mut A, event_store::prelude::RecordedEvent) -> Result<(), ApplyError>>,
}

impl<T: Actor, A: Aggregate> EventResolverRegistry<T, A> {
    pub fn get(&self, event_name: &str) -> Option<&EventResolverFn<T>> {
        let type_id = self.names.get(event_name)?;

        self.resolvers.get(type_id)
    }

    pub fn get_applier(
        &self,
        event_name: &str,
    ) -> Option<&fn(&mut A, event_store::prelude::RecordedEvent) -> Result<(), ApplyError>> {
        let type_id = self.names.get(event_name)?;

        self.appliers.get(type_id)
    }
}
