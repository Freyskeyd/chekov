use super::{EventNotification, Listener};
use crate::event::handler::Subscribe;
use crate::event_store::EventStore;
use crate::message::{ResolveAndApplyMany, StartListening};
use crate::Application;
use actix::{Actor, Handler, ResponseFuture, Supervised, SystemService};
use actix::{ActorFutureExt, Addr, AsyncContext, Context, Recipient, WrapFuture};
use event_store::prelude::RecordedEvent;
use event_store::prelude::StartFrom;
use std::collections::BTreeMap;
use tracing::{trace, Instrument};

/// Deal with Application subscriptions
pub struct SubscriberManager<A: Application> {
    listener_url: Option<String>,
    listener: Option<Addr<Listener<A>>>,
    subscribers: BTreeMap<String, Vec<Recipient<ResolveAndApplyMany>>>,
}

impl<A: Application> std::default::Default for SubscriberManager<A> {
    fn default() -> Self {
        unimplemented!()
    }
}

impl<A: Application> SubscriberManager<A> {
    pub(crate) fn new(listener_url: Option<String>) -> Self {
        Self {
            listener_url,
            listener: None,
            subscribers: BTreeMap::new(),
        }
    }

    fn add_sub(&mut self, stream: &str, sub: Recipient<ResolveAndApplyMany>) {
        let stream_id = stream.to_string();
        if let Some(subs) = self.subscribers.get_mut(&stream_id) {
            trace!("Broker: Adding to subscription list.");
            subs.push(sub);
            return;
        }

        trace!("Broker: Creating subscription list.");
        self.subscribers.insert(stream_id, vec![sub]);
    }
}

impl<A: Application> SystemService for SubscriberManager<A> {}
impl<A: Application> Supervised for SubscriberManager<A> {}

impl<A: Application> Actor for SubscriberManager<A> {
    type Context = Context<Self>;

    #[tracing::instrument(name = "SubscriberManager", skip(self, _ctx), fields(app = %A::get_name()))]
    fn started(&mut self, _ctx: &mut Self::Context) {}
}

impl<A: Application> Handler<StartListening> for SubscriberManager<A> {
    type Result = ();

    #[tracing::instrument(name = "SubscriberManager", skip(self, ctx), fields(app = %A::get_name()))]
    fn handle(&mut self, msg: StartListening, ctx: &mut Self::Context) -> Self::Result {
        let listener_url = self.listener_url.clone();
        ctx.wait(
            async {
                if let Some(listener_url) = listener_url {
                    trace!("Start listener with {}", listener_url);

                    Some(Listener::setup(listener_url).await.unwrap())
                } else {
                    None
                }
            }
            .instrument(tracing::Span::current())
            .into_actor(self)
            .map(|res, actor, _| {
                actor.listener = res;
            }),
        );
    }
}

impl<A: Application> Handler<Subscribe> for SubscriberManager<A> {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, subscription: Subscribe, _ctx: &mut Self::Context) -> Self::Result {
        let stream_uuid = subscription.stream.to_string();
        let subscription_name = format!("subscription-{}", subscription.stream);
        let recipient = subscription.recipient.clone();

        self.add_sub(&subscription.stream, subscription.resolver);

        Box::pin(async {
            let addr = EventStore::<A>::get_addr().await.unwrap();
            trace!("Subscribing from SubscriberManager");
            let _ = event_store::prelude::Subscriptions::<A::Storage>::subscribe_to_stream(
                recipient,
                event_store::prelude::SubscriptionOptions {
                    stream_uuid,
                    subscription_name,
                    start_from: StartFrom::Origin,
                    transient: false,
                },
                addr,
            )
            .await;
        })
    }
}

impl<A> Handler<EventNotification> for SubscriberManager<A>
where
    A: Application,
{
    type Result = ();

    #[tracing::instrument(name = "SubscriberManager", skip(self, subscription, ctx), fields(app = %A::get_name()))]
    fn handle(&mut self, subscription: EventNotification, ctx: &mut Self::Context) {
        trace!(
            "Fetching Related RecordedEvents for {}",
            subscription.stream_uuid
        );
        let _ = ctx.address();
        let stream_uuid = subscription.stream_uuid.clone();
        let fut = async move {
            match crate::event_store::EventStore::<A>::with_reader(
                event_store::read()
                    .stream(&subscription.stream_uuid)
                    .unwrap()
                    .from(event_store::prelude::ReadVersion::Version(
                        subscription.first_stream_version as i64,
                    ))
                    .limit(
                        (subscription.last_stream_version - subscription.first_stream_version + 1)
                            as usize,
                    ),
            )
            // TODO deal with mailbox error
            .await
            .unwrap()
            {
                Ok(events) => events
                    .into_iter()
                    .map(|mut event| {
                        // TODO big hack to have a working $all stream
                        event.stream_uuid = subscription.stream_uuid.to_string();
                        event
                    })
                    .collect::<Vec<RecordedEvent>>(),
                Err(e) => panic!("{:?}", e),
            }
        }
        .in_current_span()
        .into_actor(self)
        .map(move |res, actor, _ctx| {
            if let Some(subs) = actor.subscribers.get(&stream_uuid) {
                for sub in subs {
                    let _ = sub.try_send(ResolveAndApplyMany(res.clone()));
                }
            }
        });

        ctx.spawn(fut);
    }
}
