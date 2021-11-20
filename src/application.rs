//! Struct and Trait correlated to Application

mod builder;
mod internal;

pub use builder::ApplicationBuilder;
pub(crate) use internal::InternalApplication;

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
///     // Define that this application will use a PostgresStorage as event_store
///     type Storage = event_store::prelude::PostgresStorage;
/// }
/// ```
pub trait Application: Unpin + 'static + Send + std::default::Default {
    /// The type of storage used by the application
    type Storage: event_store::prelude::Storage;

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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::aggregates::support::MyApplication;

    #[test]
    fn application_must_have_a_name() {
        assert_eq!(
            MyApplication::get_name(),
            "chekov::tests::aggregates::support::MyApplication"
        );
    }
}
