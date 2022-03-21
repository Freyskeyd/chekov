use std::{collections::VecDeque, sync::Arc};

use actix::{Actor, Context, Handler, ResponseFuture};
use tokio::sync::Mutex;

use crate::subscriptions::SubscriptionNotification;

pub(crate) mod event;
pub(crate) mod subscriber;

pub(crate) type Tracker = Arc<Mutex<VecDeque<SubscriptionNotification>>>;
pub(crate) struct InnerSub {
    pub(crate) reference: Tracker,
}

impl Actor for InnerSub {
    type Context = Context<Self>;
}

impl Handler<SubscriptionNotification> for InnerSub {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: SubscriptionNotification, _ctx: &mut Self::Context) -> Self::Result {
        let aquire = Arc::clone(&self.reference);

        Box::pin(async move {
            let mut mutex = aquire.lock().await;
            mutex.push_back(msg);

            Ok(())
        })
    }
}
