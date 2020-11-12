use crate::error::ApplyError;
use event_store::Event;

pub trait EventApplier<E: Event> {
    fn apply(&mut self, event: &E) -> Result<(), ApplyError>;
}
