use event_store_core::storage::Storage;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::EventStore;

use super::{
    fsm::{InternalFSMState, SubscriptionFSM},
    supervisor::{Down, GoingDown, Notify, Started, SubscriptionsSupervisor},
};
use super::{SubscriptionNotification, SubscriptionOptions};
use actix::prelude::*;
use tracing::debug;

#[derive(Debug, Message)]
#[rtype("()")]
struct Connect(pub Recipient<SubscriptionNotification>, SubscriptionOptions);

#[derive(Debug, Message)]
#[rtype("()")]
struct CatchUp;

#[derive(Debug)]
pub struct Subscription<S: Storage> {
    stream_uuid: String,
    subscription_name: String,
    subscription: Arc<Mutex<SubscriptionFSM<S>>>,
    retry_interval: usize,
    supervisor: Addr<SubscriptionsSupervisor<S>>,
}

impl<S: Storage> Actor for Subscription<S> {
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

impl<S: Storage> Handler<Connect> for Subscription<S> {
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
                let recipient = msg.0;

                fsm.connect_subscriber(recipient).await;
                fsm.subscribe().await;
            }

            fsm.state
        }
        .into_actor(self)
        .map(|res, actor, ctx| {
            actor.handle_state_update(res, ctx);
        });

        Box::pin(fut)
    }
}

impl<S: Storage> Handler<CatchUp> for Subscription<S> {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _: CatchUp, _ctx: &mut Self::Context) -> Self::Result {
        debug!(
            "{} attempting to catch up {:?}",
            self.subscription_name, self.stream_uuid
        );

        let fsm_arc = self.subscription.clone();

        let fut = async move {
            let mut fsm = fsm_arc.lock().await;

            fsm.catch_up().await;

            fsm.state
        }
        .into_actor(self)
        .map(|res, actor, ctx| {
            actor.handle_state_update(res, ctx);
        });

        Box::pin(fut)
    }
}

impl<S: Storage> Handler<Notify> for Subscription<S> {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: Notify, _ctx: &mut Self::Context) -> Self::Result {
        debug!(
            "{} attempting to notify subscriber ",
            self.subscription_name
        );

        let fsm = self.subscription.clone();
        let fut = async move {
            fsm.lock().await.notify_events(msg.0).await;
        };

        Box::pin(fut)
    }
}
impl<S: Storage> Supervised for Subscription<S> {}

impl<S: Storage> Subscription<S> {
    #[must_use]
    pub fn start_with_options(
        options: &SubscriptionOptions,
        supervisor: Addr<SubscriptionsSupervisor<S>>,
        storage: Addr<EventStore<S>>,
    ) -> Addr<Self> {
        let subscription = Self {
            stream_uuid: options.stream_uuid.clone(),
            subscription_name: options.subscription_name.clone(),
            subscription: Arc::new(Mutex::new(SubscriptionFSM::with_options(options, storage))),
            retry_interval: 1_000,
            supervisor,
        };

        subscription.start()
    }

    pub async fn connect(
        addr: &Addr<Self>,
        recipient: Recipient<SubscriptionNotification>,
        options: &SubscriptionOptions,
    ) -> Result<(), ()> {
        debug!(
            "Send Connect command to subscription {}",
            options.subscription_name
        );
        // TODO handle result
        let _ = addr.send(Connect(recipient, options.clone())).await;

        Ok(())
    }

    fn handle_state_update(&mut self, state: InternalFSMState, ctx: &mut Context<Self>) {
        match state {
            InternalFSMState::Initial | InternalFSMState::Disconnected => {
                // subscribe to stream
            }
            InternalFSMState::RequestCatchUp => {
                debug!("{} catching up", self.subscription_name);
                ctx.notify(CatchUp);
            }
            InternalFSMState::Unsubscribed => {
                debug!(
                    "{} has no subscribers, shutting down",
                    self.subscription_name
                );
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{event::RecordedEvents, subscriptions::StartFrom, InMemoryStorage};

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
        let es = EventStore::builder()
            .storage(InMemoryStorage::default())
            .build()
            .await
            .unwrap()
            .start();

        let sup = SubscriptionsSupervisor::<InMemoryStorage>::from_registry();
        let dummy = Dummy {}.start();

        let options = SubscriptionOptions {
            stream_uuid: uuid::Uuid::new_v4().to_string(),
            subscription_name: "can_subscribe_test".into(),
            start_from: StartFrom::Origin,
            transient: false,
        };

        let addr = Subscription::start_with_options(&options, sup.clone(), es);

        let recip = dummy.recipient();
        let _ = Subscription::connect(&addr, recip.clone(), &options).await;
        let _ = Subscription::connect(&addr, recip, &options).await;
    }
}
