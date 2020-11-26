use super::SubscriptionOptions;
use super::{
    fsm::SubscriptionFSM,
    supervisor::{Down, GoingDown, Started, SubscriptionsSupervisor},
};
use crate::prelude::RecordedEvents;
use crate::Storage;
use actix::prelude::*;
use tracing::debug;

#[derive(Debug, Message)]
#[rtype("()")]
struct Connect(pub Recipient<RecordedEvents>, SubscriptionOptions);
pub struct Subscription<S: Storage> {
    stream_uuid: String,
    subscription_name: String,
    subscription: SubscriptionFSM,
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
    type Result = ();

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        debug!(
            "{} attempting to connect subscriber {:?}",
            self.subscription_name, msg
        );

        // Ensure not already subscribe
        if self.subscription.has_subscriber(&msg.0) {
            return ();
        }

        let fut = async move {
            // subscription.connect_subscriber(&msg.0);
            // subscription.subscribe();
            // subscription
        }
        .into_actor(self)
        .map(|_, _, _| {
            // actor.subscription = res;
        });

        ctx.wait(fut);
    }
}

impl<S: Storage> Supervised for Subscription<S> {}

impl<S: Storage> Subscription<S> {
    pub fn start_with_options(
        options: &SubscriptionOptions,
        supervisor: Addr<SubscriptionsSupervisor<S>>,
    ) -> Addr<Self> {
        let subscription = Self {
            stream_uuid: options.stream_uuid.clone(),
            subscription_name: options.subscription_name.clone(),
            subscription: SubscriptionFSM::default(),
            retry_interval: 1_000,
            supervisor,
        };

        subscription.start()
    }

    pub async fn connect(
        addr: &Addr<Self>,
        recipient: &Recipient<RecordedEvents>,
        options: &SubscriptionOptions,
    ) -> Result<(), ()> {
        let _ = addr.send(Connect(recipient.clone(), options.clone())).await;

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::prelude::InMemoryBackend;

    struct Dummy {}
    impl Actor for Dummy {
        type Context = Context<Self>;
    }

    impl Handler<RecordedEvents> for Dummy {
        type Result = ();

        fn handle(&mut self, _: RecordedEvents, _: &mut Self::Context) -> Self::Result {}
    }

    #[actix_rt::test]
    async fn can_subscribe() {
        let sup = SubscriptionsSupervisor::<InMemoryBackend>::from_registry();
        let dummy = Dummy {}.start();

        let options = SubscriptionOptions {
            stream_uuid: uuid::Uuid::new_v4().to_string(),
            subscription_name: "can_subscribe_test".into(),
        };

        let addr = Subscription::<InMemoryBackend>::start_with_options(&options, sup.clone());

        let recip = dummy.recipient();
        let _ = Subscription::connect(&addr, &recip, &options).await;
        let _ = Subscription::connect(&addr, &recip, &options).await;
    }
}
