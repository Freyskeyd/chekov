use super::state::SubscriptionState;
use super::subscriber::Subscriber;
use crate::prelude::*;
use actix::prelude::*;

#[derive(PartialEq, Debug)]
enum FSM {
    Initialized,
    Terminated,
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
    pub fn has_subscriber(&self, subscriber: &Recipient<RecordedEvents>) -> bool {
        self.data.subscribers.get(subscriber).is_some()
    }

    pub async fn notify_subscribers(&mut self) {
        println!("DO somnething");
    }

    pub async fn connect_subscriber(&mut self, subscriber: &Recipient<RecordedEvents>) {
        let addr = Subscriber {
            recipient: subscriber.clone(),
        }
        .start();

        self.data.subscribers.insert(subscriber.clone(), addr);

        if self.state != FSM::Initialized {
            self.notify_subscribers().await;
        }
    }

    pub fn subscribe(&mut self) {}
}
