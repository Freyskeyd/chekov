use serde::Serialize;

/// Represent event that can be handled by an `EventStore`
pub trait Event: Serialize {
    /// Return a static str which define the event type
    ///
    /// This str must be as precise as possible.
    fn event_type(&self) -> &'static str;
}

mod unsaved;
mod recorded;

#[cfg(test)]
mod test;
