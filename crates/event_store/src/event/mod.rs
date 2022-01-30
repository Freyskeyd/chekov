use actix::Message;

pub use crate::core::event::Event;
pub use crate::core::event::RecordedEvent;
pub use crate::core::event::UnsavedEvent;
pub use crate::core::event::UnsavedEventError;

#[derive(Debug, Clone, Message)]
#[rtype("()")]
pub struct RecordedEvents {
    pub(crate) events: Vec<RecordedEvent>,
}
