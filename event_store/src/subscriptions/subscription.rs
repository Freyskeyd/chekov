use std::sync::Arc;
use tokio::sync::Mutex;

use crate::prelude::EventBus;

use super::{
    fsm::SubscriptionFSM,
    supervisor::{Down, GoingDown, Notify, Started, SubscriptionsSupervisor},
};
use super::{SubscriptionNotification, SubscriptionOptions};
use actix::prelude::*;
use tracing::debug;

#[derive(Debug, Message)]
#[rtype("()")]
struct Connect(pub Recipient<SubscriptionNotification>, SubscriptionOptions);

#[derive(Debug)]
pub struct Subscription<S: EventBus> {
    stream_uuid: String,
    subscription_name: String,
    subscription: Arc<Mutex<SubscriptionFSM>>,
    retry_interval: usize,
    supervisor: Addr<SubscriptionsSupervisor<S>>,
}

impl<S: EventBus> Actor for Subscription<S> {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        self.supervisor.do_send(Started);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.supervisor.do_send(GoingDown);

        Running::Stop
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        self.supervisor.do_send(Down);
    }
}

impl<S: EventBus> Subscription<S> {}

impl<S: EventBus> Handler<Connect> for Subscription<S> {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        debug!(
            "{} attempting to connect subscriber {:?}",
            self.subscription_name, msg.1.stream_uuid
        );

        let fsm_arc = self.subscription.clone();

        let fut = async move {
            let mut fsm = fsm_arc.lock().await;

            // Ensure not already subscribe
            if !fsm.has_subscriber() {
                let recipient = msg.0.clone();

                fsm.connect_subscriber(&recipient).await;
            }
            // subscription.connect_subscriber(&msg.0);
            // subscription.subscribe();
            // subscription
        }
        .into_actor(self)
        .map(|_, _, _| {
            // actor.subscription = res;
        });

        Box::pin(fut)
    }
}

impl<S: EventBus> Handler<Notify> for Subscription<S> {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Notify, _ctx: &mut Self::Context) -> Self::Result {
        debug!(
            "{} attempting to notify subscriber ",
            self.subscription_name
        );

        let fsm = self.subscription.clone();
        let fut = async move {
            fsm.lock().await.notify_events(msg.0).await;

            // subscription.connect_subscriber(&msg.0);
            // subscription.subscribe();
            // subscription
        }
        .into_actor(self)
        .map(|_, _, _| {
            // actor.subscription = res;
        });

        Box::pin(fut)
    }
}
impl<S: EventBus> Supervised for Subscription<S> {}

impl<S: EventBus> Subscription<S> {
    pub fn start_with_options(
        options: &SubscriptionOptions,
        supervisor: Addr<SubscriptionsSupervisor<S>>,
    ) -> Addr<Self> {
        let subscription = Self {
            stream_uuid: options.stream_uuid.clone(),
            subscription_name: options.subscription_name.clone(),
            subscription: Arc::new(Mutex::new(SubscriptionFSM::default())),
            retry_interval: 1_000,
            supervisor,
        };

        subscription.start()
    }

    pub async fn connect(
        addr: &Addr<Self>,
        recipient: &Recipient<SubscriptionNotification>,
        options: &SubscriptionOptions,
    ) -> Result<(), ()> {
        // TODO handle result
        let _ = addr.send(Connect(recipient.clone(), options.clone())).await;

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{event::RecordedEvents, prelude::InMemoryEventBus};

    struct Dummy {}
    impl Actor for Dummy {
        type Context = Context<Self>;
    }

    impl Handler<SubscriptionNotification> for Dummy {
        type Result = Result<(), ()>;

        fn handle(&mut self, _: SubscriptionNotification, _: &mut Self::Context) -> Self::Result {
            Ok(())
        }
    }

    impl Handler<RecordedEvents> for Dummy {
        type Result = ();

        fn handle(&mut self, _: RecordedEvents, _: &mut Self::Context) -> Self::Result {}
    }

    #[actix::test]
    async fn can_subscribe() {
        let sup = SubscriptionsSupervisor::<InMemoryEventBus>::from_registry();
        let dummy = Dummy {}.start();

        let options = SubscriptionOptions {
            stream_uuid: uuid::Uuid::new_v4().to_string(),
            subscription_name: "can_subscribe_test".into(),
        };

        let addr = Subscription::<InMemoryEventBus>::start_with_options(&options, sup.clone());

        let recip = dummy.recipient();
        let _ = Subscription::connect(&addr, &recip, &options).await;
        let _ = Subscription::connect(&addr, &recip, &options).await;
    }
}
