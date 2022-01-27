use std::collections::VecDeque;

use crate::event::RecordedEvent;
use crate::prelude::ReadVersion;
use crate::storage::reader::Reader;
use crate::EventStore;

use super::subscriber::Subscriber;
use super::SubscriptionNotification;
use super::{state::SubscriptionState, SubscriptionOptions};
use actix::prelude::*;
use event_store_core::storage::Storage;
use tracing::debug;

/// connect_subscriber -> subscribe
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

#[derive(Debug)]
pub struct SubscriptionFSM<S: Storage> {
    pub(crate) data: SubscriptionState<S>,
    pub(crate) state: InternalFSMState,
}

impl<S: Storage> SubscriptionFSM<S> {
    pub fn with_options(options: &SubscriptionOptions, storage: Addr<EventStore<S>>) -> Self {
        Self {
            data: SubscriptionState {
                subscriber: None,
                storage,
                stream_uuid: options.stream_uuid.clone(),
                start_from: options.start_from,
                subscription_name: options.subscription_name.clone(),
                last_received: 0,
                last_sent: 0,
                last_ack: 0,
                queue: VecDeque::new(),
                transient: options.transient,
                in_flight_event_numbers: Vec::new(),
            },
            state: InternalFSMState::default(),
        }
    }

    pub fn has_subscriber(&self) -> bool {
        self.data.subscriber.is_some()
    }

    pub async fn notify_subscribed(&mut self) {
        debug!(
            "Executing notify_subscribed for {}",
            self.data.subscription_name
        );

        if let Some(subscriber) = &self.data.subscriber {
            let _ = subscriber.notify_subscribed().await;
        }
    }

    pub async fn notify_events(&mut self, events: Vec<RecordedEvent>) {
        if let Some(subscriber) = &self.data.subscriber {
            let _ = subscriber
                .recipient
                .send(SubscriptionNotification::Events(events))
                .await;
        }
    }

    // State: *
    // Used to connect a subscriber to this subscription.
    #[tracing::instrument(skip(self, subscriber))]
    pub async fn connect_subscriber(&mut self, subscriber: Recipient<SubscriptionNotification>) {
        debug!(
            "Executing connect_subscriber for {}",
            self.data.subscription_name
        );

        let sub = Subscriber::with_recipient(subscriber);

        // Monitor subscriber
        self.data.subscriber = Some(sub);
        debug!("Current state is {:?}", self.state);
        // TODO: notify subscribers for events
        if self.state != InternalFSMState::Initial {
            // notify_subscribed
            self.notify_subscribed().await;
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn subscribe(&mut self) {
        debug!("Executing subscribe for {}", self.data.subscription_name);

        match self.state {
            InternalFSMState::Initial => {
                if self.data.transient {
                    self.data.reset_event_tracking();
                    if self.subscribe_to_events().await.is_ok() {
                        let start_from: i64 = self.data.start_from.into();
                        debug!("Start from is : {}", start_from);
                        self.data.last_received = start_from;
                        self.data.last_sent = start_from;
                        self.data.last_ack = start_from;

                        self.notify_subscribed().await;
                        self.state = InternalFSMState::RequestCatchUp;
                        // go to request_catch_up ->
                    } else {
                        // retry after a delay
                        self.state = InternalFSMState::Initial;
                    }
                }
            }
            InternalFSMState::Disconnected => {}
            _ => {}
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn catch_up(&mut self) {
        debug!("Executing catch_up for {}", self.data.subscription_name);
        let _ = match self.state {
            InternalFSMState::RequestCatchUp => self.catch_up_from_stream().await,
            InternalFSMState::Subscribed => {
                self.state = InternalFSMState::RequestCatchUp;

                Ok(())
            }
            _ => Ok(()),
        };
    }

    async fn initialize(&mut self) {}

    #[tracing::instrument(skip(self))]
    async fn subscribe_to_events(&mut self) -> Result<(), ()> {
        // PubSub::subscribe
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn catch_up_from_stream(&mut self) -> Result<(), ()> {
        debug!(
            "Executing catch_up_from_stream for {}",
            self.data.subscription_name
        );
        if self.data.queue.is_empty() {
            match self.read_stream_forward().await {
                Ok(events) if events.is_empty() => {
                    if self.data.last_sent == self.data.last_received {
                        // Subscriber is up-to-date with latest published events
                        self.state = InternalFSMState::Subscribed;
                        Ok(())
                    } else {
                        // Need to catch-up with events published while catching up
                        self.state = InternalFSMState::RequestCatchUp;

                        Ok(())
                    }
                }

                Ok(events) => {
                    // apply events and check queue size
                    self.enqueue_events(events);
                    self.notify_subscribers().await;
                    // Wait for ack
                    self.state = InternalFSMState::CatchingUp;
                    Ok(())
                }

                Err(_) => {
                    self.state = InternalFSMState::Subscribed;

                    Ok(())
                }
            }
        } else {
            self.state = InternalFSMState::CatchingUp;
            Ok(())
        }
    }

    #[tracing::instrument(skip(self))]
    async fn read_stream_forward(&mut self) -> Result<Vec<RecordedEvent>, ()> {
        debug!(
            "Executing read_stream_forward for {}",
            self.data.subscription_name
        );
        let SubscriptionState {
            storage,
            stream_uuid,
            last_sent,
            ..
        } = &self.data;

        let limit = last_sent + 1;

        Reader::default()
            .stream(stream_uuid)
            .unwrap()
            .from(ReadVersion::Version(limit))
            .execute(storage.clone())
            .await
            .map_err(|_| ())
    }

    #[tracing::instrument(skip(self))]
    async fn notify_subscribers(&mut self) {
        debug!(
            "Executing notify_subscribers for {}",
            self.data.subscription_name
        );
        while let Some(to_notify @ RecordedEvent { event_number, .. }) = self.data.queue.pop_front()
        {
            if let Some(subscriber) = self.data.subscriber.as_mut() {
                subscriber.track_in_flight(to_notify);

                self.track_sent(event_number);
                self.track_last_received(event_number);
            }
        }

        if let Some(subscriber) = self.data.subscriber.as_mut() {
            subscriber.send_queued_events().await;
        }
    }

    #[tracing::instrument(skip(self, events))]
    fn enqueue_events(&mut self, mut events: Vec<RecordedEvent>) {
        debug!(
            "Executing enqueue_events for {} with {} events",
            self.data.subscription_name,
            events.len()
        );
        let last_received = if let Some(RecordedEvent { event_number, .. }) =
            events.iter().max_by_key(|e| e.event_number)
        {
            self.data.last_received.max(*event_number)
        } else {
            self.data.last_received
        };

        events.sort_by_key(|e| e.event_number);

        self.data.queue.extend(events);
        self.data.last_received = last_received;
    }

    #[tracing::instrument(skip(self))]
    fn track_sent(&mut self, event_number: i64) {
        debug!("Executing track_sent for {}", self.data.subscription_name);
        self.data.last_sent = std::cmp::max(self.data.last_sent, event_number);
        // TODO: Improve this part to be more efficient
        self.data.in_flight_event_numbers.push(event_number);
        self.data.in_flight_event_numbers.sort();
        self.data.in_flight_event_numbers.dedup();
    }

    #[tracing::instrument(skip(self))]
    fn track_last_received(&mut self, event_number: i64) {
        debug!(
            "Executing track_last_received for {}",
            self.data.subscription_name
        );
        self.data.last_received = std::cmp::max(self.data.last_received, event_number);
        // TODO: Improve this part to be more efficient
        self.data.in_flight_event_numbers.push(event_number);
        self.data.in_flight_event_numbers.sort();
        self.data.in_flight_event_numbers.dedup();
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub(crate) enum InternalFSMState {
    Initial,
    RequestCatchUp,
    CatchingUp,
    Subscribed,
    MaxCapacity,
    Disconnected,
    Unsubscribed,
}

impl Default for InternalFSMState {
    fn default() -> Self {
        Self::Initial
    }
}
