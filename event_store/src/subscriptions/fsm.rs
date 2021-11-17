use crate::event::RecordedEvents;

use super::state::SubscriptionState;
use super::subscriber::Subscriber;
use super::SubscriptionNotification;
use actix::prelude::*;

#[derive(PartialEq, Debug)]
enum FSM {
    Initialized,
    Terminated,
    RequestCatchUp,
}

impl std::default::Default for FSM {
    fn default() -> Self {
        Self::Initialized
    }
}

#[derive(Default, Debug)]
pub struct SubscriptionFSM {
    data: SubscriptionState,
    state: FSM,
}

impl SubscriptionFSM {
    pub fn has_subscriber(&self) -> bool {
        self.data.subscriber.is_some()
    }

    pub async fn notify_subscribers(&mut self) {
        if let Some(subscriber) = &self.data.subscriber {
            let _ = subscriber.send(SubscriptionNotification::Subscribed).await;
        }
    }

    pub async fn notify_events(&mut self, events: RecordedEvents) {
        if let Some(subscriber) = &self.data.subscriber {
            let _ = subscriber
                .send(SubscriptionNotification::Events(events.events))
                .await;
        }
    }

    pub async fn connect_subscriber(&mut self, subscriber: &Recipient<SubscriptionNotification>) {
        let addr = Subscriber {
            recipient: subscriber.clone(),
        }
        .start();

        self.data.subscriber = Some(addr);

        if self.state == FSM::Initialized {
            self.notify_subscribers().await;
        }
    }

    pub async fn subscribe(&mut self) {
        match self.state {
            FSM::Initialized => {
                self.initialize().await;
            }
            _ => todo!(),
        }
    }

    async fn initialize(&mut self) {
        // self.data.queue_size = 0;

        self.subscribe_to_events().await;
        self.notify_subscribers().await;

        self.state = FSM::RequestCatchUp;
    }

    async fn subscribe_to_events(&mut self) {
        // PubSub.subscribe(event_store, stream_uuid)
        // self.data.last_received = self.data.start_from;
        // self.data.last_sent = self.data.start_from;
        // self.data.last_ack = self.data.start_from;
    }
}
