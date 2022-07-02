use std::time::Duration;

use chekov_api::{ChekovClient, Update};
use tonic::{
    transport::{Channel, Uri},
    Streaming,
};

pub struct Connection {
    target: Uri,
    state: State,
}

impl Connection {
    pub fn new(target: Uri) -> Self {
        Self {
            target,
            state: State::Disconnected(Duration::from_secs(0)),
        }
    }

    pub async fn next_update(&mut self) {}
}

enum State {
    Connected {
        client: ChekovClient<Channel>,
        stream: Box<Streaming<Update>>,
    },
    Disconnected(Duration),
}
