use actix::Message;

pub use crate::core::event::Event;
pub use crate::core::event::RecordedEvent;
pub use crate::core::event::UnsavedEvent;

#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct RecordedEvents {
    pub(crate) events: Vec<RecordedEvent>,
}
