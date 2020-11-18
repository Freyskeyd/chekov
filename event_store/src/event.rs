use serde::Serialize;

/// Represent event that can be handled by an `EventStore`
pub trait Event: Serialize + Send + std::convert::TryFrom<RecordedEvent> {
    /// Return a static str which define the event type
    ///
    /// This str must be as precise as possible.
    fn event_type(&self) -> &'static str;
    fn event_type_from_str() -> &'static str {
        std::any::type_name::<Self>()
    }
}

mod recorded;
mod unsaved;
pub use recorded::RecordedEvent;
pub use recorded::RecordedEvents;
pub use unsaved::ParseEventError;
pub use unsaved::UnsavedEvent;

#[cfg(test)]
mod test;
