use super::*;
use crate::error::HandleError;

#[doc(hidden)]
pub type EventHandlerFn<A> = fn(&mut A, RecordedEvent) -> BoxFuture<Result<(), HandleError>>;

#[doc(hidden)]
pub struct EventHandlerResolverRegistry<E: EventHandler> {
    pub names: BTreeMap<&'static str, TypeId>,
    pub handlers: BTreeMap<TypeId, EventHandlerFn<E>>,
}

impl<E: EventHandler> EventHandlerResolverRegistry<E> {
    pub fn get(&self, event_name: &str) -> Option<&EventHandlerFn<E>> {
        let type_id = self.names.get(event_name)?;

        self.handlers.get(type_id)
    }
}
