use std::{collections::VecDeque, sync::Arc};

use actix::prelude::*;

use crate::event::RecordedEvent;

use super::SubscriptionNotification;

#[derive(Debug)]
pub struct Subscriber {
    pub recipient: Recipient<SubscriptionNotification>,
    pub(crate) in_flight: VecDeque<Arc<RecordedEvent>>,
    last_sent: u64,
}

impl Actor for Subscriber {
    type Context = Context<Self>;
}

impl Subscriber {
    pub(crate) fn with_recipient(recipient: Recipient<SubscriptionNotification>) -> Self {
        Self {
            recipient,
            in_flight: VecDeque::default(),
            last_sent: 0,
        }
    }

    pub(crate) async fn notify_subscribed(&self) -> Result<Result<(), ()>, MailboxError> {
        self.recipient
            .send(SubscriptionNotification::Subscribed)
            .await
    }

    pub(crate) fn track_in_flight(&mut self, event: Arc<RecordedEvent>) {
        self.last_sent = event.event_number;
        self.in_flight.push_front(event);
    }

    pub(crate) async fn send_queued_events(&mut self) {
        let mut events: Vec<_> = self.in_flight.clone().into();

        events.reverse();

        // TODO: How to handle notification failure ?
        let _ = self
            .recipient
            .send(SubscriptionNotification::Events(Arc::new(events)))
            .await;
    }
}
