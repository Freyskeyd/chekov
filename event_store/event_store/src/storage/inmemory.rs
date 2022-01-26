use event_store_core::{
    event_bus::{BoxedStream, EventBus, EventBusMessage},
    storage::Storage,
};
use tokio::sync::mpsc::{self};

use crate::prelude::InMemoryBackend;

use super::event_bus::InMemoryEventBus;

/// InMemory storage used for tests mostly
#[derive(Default, Debug)]
pub struct InMemoryStorage {
    backend: InMemoryBackend,
    event_bus: InMemoryEventBus,
}

impl InMemoryStorage {
    pub async fn initiate() -> Result<Self, ()> {
        Ok(Self::default())
    }
}

impl Storage for InMemoryStorage {
    type Backend = InMemoryBackend;
    type EventBus = InMemoryEventBus;

    fn storage_name() -> &'static str {
        "InMemory"
    }

    fn direct_channel(&mut self, notifier: mpsc::UnboundedSender<EventBusMessage>) {
        self.backend.notifier = Some(notifier);
    }

    fn create_stream(&mut self) -> BoxedStream {
        self.backend.notifier = Some(self.event_bus.sender.clone());
        self.event_bus.create_stream()
    }

    fn backend(&mut self) -> &mut Self::Backend {
        &mut self.backend
    }
}
