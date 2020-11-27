use super::message;
use crate::application::Application;
use actix::prelude::*;
use actix_interop::{with_ctx, FutureInterop};
use sqlx::postgres::PgNotification;
use std::collections::HashMap;
use tracing::trace;
use uuid::Uuid;

pub struct Subscriber {
    _stream: String,
    _subscriber: Recipient<message::EventEnvelope>,
}

impl Subscriber {
    pub(crate) fn new(subscriber: Recipient<message::EventEnvelope>, stream: &str) -> Addr<Self> {
        trace!("Create a new Subscriber for {}", stream);
        Subscriber::create(move |_ctx| Subscriber {
            _stream: stream.to_string(),
            _subscriber: subscriber,
        })
    }
}

impl actix::Actor for Subscriber {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        // let current_address = ctx.address();
        // self.subscribe_sync::<BrokerType, EventNotification>(ctx);
        // ctx.wait(
        //     async {
        //         let res = SubscriberManager::from_registry()
        //             .send(Subscribe {
        //                 subscriber_id: Uuid::new_v4(),
        //                 subscriber: current_address.recipient(),
        //             })
        //             .await;

        //         println!("{:?}", res);
        //     }
        //     .into_actor(self),
        // );
    }
}

impl actix::Handler<EventNotification> for Subscriber {
    type Result = ();

    fn handle(&mut self, e: EventNotification, _ctx: &mut Self::Context) -> Self::Result {
        println!("Subscriber received {:?}", e);
    }
}

impl actix::Handler<message::EventEnvelope> for Subscriber {
    type Result = Result<usize, ()>;

    fn handle(&mut self, _: message::EventEnvelope, _ctx: &mut Self::Context) -> Self::Result {
        Err(())
    }
}

use fnv::FnvHasher;
use std::any::Any;
use std::hash::BuildHasherDefault;

type TypeMap<A> = HashMap<String, A, BuildHasherDefault<FnvHasher>>;

use std::any::TypeId;

pub struct SubscriberManager<A: Application> {
    listener: Option<Addr<Listener<A>>>,
    _event_store: actix::Addr<event_store::EventStore<A::Storage>>,
    eventmapper: HashMap<String, TypeId>,
    // _subscribers: HashMap<String, Vec<Recipient<message::EventEnvelope>>>,
    sub_map: TypeMap<Vec<(TypeId, Box<dyn Any>)>>,
    // msg_map: TypeMap<Box<dyn Any>>,
}

impl<A: Application> std::default::Default for SubscriberManager<A> {
    fn default() -> Self {
        unimplemented!()
    }
}

impl<A: Application> SubscriberManager<A> {
    pub(crate) fn new(
        event_store: Addr<event_store::EventStore<A::Storage>>,
        eventmapper: HashMap<String, TypeId>,
    ) -> Self {
        Self {
            listener: None,
            _event_store: event_store,
            eventmapper,
            sub_map: TypeMap::default(),
            // msg_map: TypeMap::default(),
        }
    }

    fn add_sub<M: crate::event_handler::BrokerMsg>(
        &mut self,
        stream: &str,
        sub: Recipient<M>,
        _id: &str,
    ) {
        let msg_id = TypeId::of::<M>();
        let stream_id = stream.to_string();
        let boxed = Box::new(sub);
        if let Some(subs) = self.sub_map.get_mut(&stream_id) {
            trace!("Broker: Adding to {:?} subscription list.", msg_id);
            subs.push((msg_id, boxed));
            return;
        }

        trace!("Broker: Creating {:?} subscription list.", msg_id);
        self.sub_map.insert(stream_id, vec![(msg_id, boxed)]);
    }
}

impl<A: Application> actix::registry::ArbiterService for SubscriberManager<A> {}
impl<A: Application> actix::Supervised for SubscriberManager<A> {}

impl<A: Application> actix::Actor for SubscriberManager<A> {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.wait(
            async {
                trace!("Create a new SubscriberManager");
                let addr = Listener::setup().await.unwrap();
                addr
            }
            .into_actor(self)
            .map(|res, actor, _| {
                actor.listener = Some(res);
            }),
        );
    }
}

impl<A: Application, M: crate::event_handler::BrokerMsg + std::fmt::Debug>
    actix::Handler<crate::event_handler::Subscribe<M>> for SubscriberManager<A>
{
    type Result = ();
    fn handle(
        &mut self,
        subscription: crate::event_handler::Subscribe<M>,
        _ctx: &mut Self::Context,
    ) {
        self.add_sub(&subscription.0, subscription.1, &subscription.2);
    }
}

impl<A> actix::Handler<EventNotification> for SubscriberManager<A>
where
    A: Application,
{
    type Result = ();
    fn handle(&mut self, subscription: EventNotification, ctx: &mut Self::Context) {
        trace!(
            "Fetching Related RecordedEvents for {}",
            subscription.stream_uuid
        );

        let fut = async move {
            let result = match event_store::read()
                .stream(&subscription.stream_uuid)
                .unwrap()
                .from(event_store::prelude::ReadVersion::Version(
                    subscription.first_stream_version as i64,
                ))
                .limit(
                    (subscription.last_stream_version - subscription.first_stream_version + 1)
                        as usize,
                )
                .execute_async::<A::Storage>()
                .await
            {
                Ok(events) => events,
                Err(_) => panic!(""),
            };

            // let k = critical_section::<Self, _>(async {
            for event in result {
                let _: Option<Vec<(TypeId, Box<dyn Any>)>> = with_ctx(|actor: &mut Self, _| {
                    match actor.eventmapper.get(&event.event_type) {
                        Some(type_id) => match actor.sub_map.get_mut(&event.stream_uuid) {
                            Some(subs) => {
                                Some(subs.drain(..).filter(|(t_id, _)| t_id == type_id).collect())
                            }
                            None => None,
                        },
                        None => None,
                    }
                });
            }
            // "OKK".to_string()
            // })
            // .await;
        }
        .interop_actor(self)
        .map(|_res, _actor, _ctx| {});
        ctx.spawn(fut);

        // if let Some(subs) = self.sub_map.get_mut(&subscription.stream_uuid) {
        //     trace!(
        //         "SubscriberManager: Found subscription list for {:?}.",
        //         subscription.stream_uuid
        //     );

        //     subs.drain(..).for_each(|(id, r)| {
        //     });
        // }
        // let subs = subs
        //     .drain(..)
        //     .filter_map(|(id, s)| {
        //         if let Ok(rec) = s.downcast::<Recipient<EventNotification>>() {
        //             Some((id, rec))
        //         } else {
        //             None
        //         }
        //     })
        //     .map(|(id, s)| (id, *s))
        //     .collect();

        // subs.drain(..)
        //     // .filter_map(|(id, s)| {
        //     //     if id == msg.1 {
        //     //         Some((id, s))
        //         if s.do_send(msg.0.clone()).is_ok() {
        //             Some((id, s))
        //         } else {
        //             None
        //         }
        //     })
        // .for_each(|(id, s)| self.add_sub::<M>(s, id));
        // Some(subs)
        // self.sub_map.iter().filter(|(stream, _)| stream == subscription.stream_uuid).
    }
}

impl<A: Application> actix::Handler<Subscribe> for SubscriberManager<A> {
    type Result = ();
    fn handle(&mut self, subscription: Subscribe, _ctx: &mut Self::Context) {
        trace!(
            "Registering {} as subscriber on {}",
            subscription.subscriber_id,
            "$all"
        );
        // self._subscribers
        //     .entry("$all".into())
        //     .or_insert(Vec::new())
        //     .push(subscription.subscriber);
    }
}

pub struct Listener<A: Application> {
    _phantom: std::marker::PhantomData<A>,
    pub listening: String,
}

impl<A: Application> actix::Actor for Listener<A> {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        trace!("Created a Listener instance");
    }
}

impl<A: Application> Listener<A> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            listening: String::new(),
        }
    }

    #[must_use]
    pub async fn setup() -> Result<Addr<Self>, ()> {
        let mut listener = sqlx::postgres::PgListener::connect(
            "postgresql://postgres:postgres@localhost/event_store_bank",
        )
        .await
        .unwrap();
        listener.listen("events").await.unwrap();

        Ok(Listener::create(move |ctx| {
            ctx.add_stream(listener.into_stream());

            Listener::new()
        }))
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Ping;

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct Notif(PgNotification);

#[derive(Message)]
#[rtype(result = "()")]
struct Subscribe {
    subscriber_id: Uuid,
    _subscriber: Recipient<message::EventEnvelope>,
}
// type BrokerType = crate::broker::broker::SystemBroker;
// use crate::broker::issue::BrokerIssue;
// use crate::broker::subscribe::BrokerSubscribe;

impl<A: Application> actix::StreamHandler<Result<PgNotification, sqlx::Error>> for Listener<A> {
    fn handle(&mut self, item: Result<PgNotification, sqlx::Error>, _ctx: &mut Self::Context) {
        match item {
            Ok(m) => match EventNotification::try_from(m.payload()) {
                Ok(event) => SubscriberManager::<A>::from_registry().do_send(event),
                _ => {}
            },
            _ => {}
        };
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        ctx.stop();
    }
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct EventNotification {
    stream_id: i32,
    stream_uuid: String,
    first_stream_version: i32,
    last_stream_version: i32,
}

trait IsEmptyResult {
    fn is_empty_result<E>(self, err: E) -> Result<Self, E>
    where
        Self: std::marker::Sized;
}

impl<'a> IsEmptyResult for &'a str {
    fn is_empty_result<E>(self, err: E) -> Result<Self, E> {
        if self.is_empty() {
            Err(err)
        } else {
            Ok(self)
        }
    }
}

use std::convert::TryFrom;

impl<'a> TryFrom<&'a str> for EventNotification {
    type Error = &'static str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let elements = value.splitn(4, ',').collect::<Vec<&str>>();

        let mut through = elements.iter();

        let stream_uuid = through
            .next()
            .ok_or("unable to parse stream_uuid")?
            .is_empty_result("unable to parse stream_uuid")?
            .to_owned();

        let stream_id = through
            .next()
            .ok_or("unable to parse stream_id")?
            .parse::<i32>()
            .or(Err("unable to convert stream_id to i32"))?;

        let first_stream_version = through
            .next()
            .ok_or("unable to parse first_stream_version")?
            .parse::<i32>()
            .or(Err("unable to convert first_stream_version to i32"))?;

        let last_stream_version = through
            .next()
            .ok_or("unable to parse last_stream_version")?
            .parse::<i32>()
            .or(Err("unable to convert last_stream_version to i32"))?;

        Ok(Self {
            stream_uuid,
            stream_id,
            first_stream_version,
            last_stream_version,
        })
    }
}
