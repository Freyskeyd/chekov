//! Struct and Trait correlated to Application

mod builder;
mod internal;

use std::{any::TypeId, collections::HashMap};

pub use builder::ApplicationBuilder;
pub(crate) use internal::InternalApplication;

use crate::{event::BoxedResolver, Event, EventResolver, SubscriberManager};

/// Application are high order logical seperator.
///
/// A chekov application is a simple entity that will represent a single and unique domain
/// application.
///
/// It is used to separate and allow multiple chekov application on a single runtime.
///
/// It's also used to define the typology of the application, like which storage will be used or
/// how to resolve the eventbus's event
///
/// The Application isn't instanciate at all and it's useless to define fields on it. It just act
/// as a type holder for the entier system.
///
/// ```rust
/// #[derive(Default)]
/// struct DefaultApp {}
///
/// impl chekov::Application for DefaultApp {
///     // Define that this application will use a PostgresBackend as event_store
///     type Storage = event_store::prelude::PostgresBackend;
///     // Define that this application will use a DefaultEventResolver as event_resolver
///     type EventResolver = chekov::application::DefaultEventResolver<Self>;
/// }
/// ```
pub trait Application: Unpin + 'static + Send + std::default::Default {
    /// The type of storage used by the application
    type Storage: event_store::prelude::Storage;

    /// The type of the event_resolver, see EventResolver documentation to know more about it.
    type EventResolver: crate::EventResolver<Self>;

    /// Used to initiate the launch of the application
    ///
    /// It will just return an ApplicationBuilder which will take care of everything.
    fn with_default() -> ApplicationBuilder<Self> {
        ApplicationBuilder::<Self>::default()
    }

    /// Returns the logical name of the application
    /// Mostly used for logs and debugging.
    ///
    /// You can configure it or use the default value which is the struct full qualified name.
    fn get_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[derive(Default)]
pub struct DefaultEventResolver<A: Application> {
    tty_str: HashMap<&'static str, TypeId>,
    resolvers: HashMap<TypeId, BoxedResolver<A>>,
}

impl<A: Application> EventResolver<A> for DefaultEventResolver<A> {
    fn resolve(
        &self,
        notify: actix::Addr<SubscriberManager<A>>,
        event_name: &str,
        event: event_store::prelude::RecordedEvent,
    ) {
        if let Some(ty) = self.tty_str.get(event_name) {
            if let Some(formatter) = self.resolvers.get(ty) {
                let _ = (formatter)(event, notify);
            }
        }
    }
}

impl<A: Application> DefaultEventResolver<A> {
    pub fn register<'de, E: Event + Clone + serde::Deserialize<'de> + 'static>(mut self) -> Self {
        let (type_string, resolver): (Vec<&'static str>, _) = E::register();

        let tty = TypeId::of::<E>();

        if self.resolvers.insert(tty, resolver).is_some() {
            panic!("Resolver already defined for this type");
        }

        type_string.into_iter().for_each(|s| {
            self.tty_str.insert(s, tty);
        });

        self
    }
}
