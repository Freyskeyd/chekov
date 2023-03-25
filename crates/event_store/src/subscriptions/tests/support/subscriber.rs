use std::{collections::VecDeque, sync::Arc};

use actix::{Actor, Addr};
use tokio::sync::Mutex;

use crate::subscriptions::SubscriptionNotification;

use super::{InnerSub, Tracker};

pub struct SubscriberFactory {}

impl SubscriberFactory {
    pub fn setup() -> (Tracker, Addr<InnerSub>) {
        let tracker: Arc<Mutex<VecDeque<SubscriptionNotification>>> =
            Arc::new(Mutex::new(VecDeque::new()));

        let addr = InnerSub {
            reference: Arc::clone(&tracker),
        }
        .start();

        (tracker, addr)
    }
}
