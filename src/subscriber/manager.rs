use super::{listener, EventNotification, Listener};
use crate::event::handler::Subscribe;
use crate::message::{ResolveAndApplyMany, StartListening};
use crate::Application;
use actix::{Actor, Handler, Supervised, SystemService};
use actix::{ActorFutureExt, Addr, AsyncContext, Context, Recipient, WrapFuture};
use event_store::prelude::RecordedEvent;
use event_store::EventStore;
use std::any::TypeId;
use std::collections::BTreeMap;
use std::collections::HashMap;
use tracing::{trace, Instrument};

/// Deal with Application subscriptions
pub struct SubscriberManager<A: Application> {
    listener_url: String,
    listener: Option<Addr<Listener<A>>>,
    subscribers: BTreeMap<String, Vec<Recipient<ResolveAndApplyMany>>>,
}

impl<A: Application> std::default::Default for SubscriberManager<A> {
    fn default() -> Self {
        unimplemented!()
    }
}

impl<A: Application> SubscriberManager<A> {
    pub(crate) fn new(listener_url: String) -> Self {
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

    #[tracing::instrument(name = "SubscriberManager", skip(self, ctx), fields(app = %A::get_name()))]
    fn started(&mut self, ctx: &mut Self::Context) {}
}

impl<A: Application> Handler<StartListening> for SubscriberManager<A> {
    type Result = ();

    #[tracing::instrument(name = "SubscriberManager", skip(self, ctx), fields(app = %A::get_name()))]
    fn handle(&mut self, msg: StartListening, ctx: &mut Self::Context) -> Self::Result {
        let listener_url = self.listener_url.clone();
        ctx.wait(
            async {
                trace!("Start listener with {}", listener_url);
                Listener::setup(listener_url).await.unwrap()
            }
            .instrument(tracing::Span::current())
            .into_actor(self)
            .map(|res, actor, _| {
                actor.listener = Some(res);
            }),
        );
    }
}
impl<A: Application> Handler<Subscribe> for SubscriberManager<A> {
    type Result = ();
    fn handle(&mut self, subscription: Subscribe, _ctx: &mut Self::Context) {
        self.add_sub(&subscription.0, subscription.1);
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
