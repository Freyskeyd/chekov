use crate::message::EventEnvelope;
use crate::subscriber::Subscriber;
// use crate::Chekov;
use actix::prelude::*;
use log::trace;

use crate::Application;

pub struct EventHandlerBuilder<E: EventHandler> {
    pub(crate) handler: E,
    pub(crate) name: String,
}

impl<E> EventHandlerBuilder<E>
where
    E: EventHandler,
{
    pub fn new(handler: E) -> Self {
        Self {
            handler,
            name: std::any::type_name::<E>().into(),
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.into();

        self
    }

    pub async fn register<A: crate::Application>(self) {
        EventHandlerInstance::<A, E>::from_builder(self);
    }
}

pub trait Listening {
    // fn started(ctx: &mut actix::Context<EventHandlerInstance<Self>>)
    // where
    //     Self: EventHandler,
    // {
    // }
}

pub trait BrokerMsg: Message<Result = ()> + Send + Clone + 'static {}
impl<M> BrokerMsg for M where M: Message<Result = ()> + Send + Clone + 'static {}

pub trait EventHandler: Listening + Sized + std::marker::Unpin + 'static {
    fn builder(self) -> EventHandlerBuilder<Self> {
        EventHandlerBuilder::new(self)
    }
    fn started<A: Application>(&mut self, _ctx: &mut actix::Context<EventHandlerInstance<A, Self>>)
    where
        Self: EventHandler,
    {
    }
    fn listen<A: Application, M: BrokerMsg + event_store::Event + std::fmt::Debug>(
        &self,
        ctx: &mut actix::Context<EventHandlerInstance<A, Self>>,
    )
    // ) where
    // EventHandlerInstance<Self>: actix::Handler<M>,
    // <EventHandlerInstance<Self> as Actor>::Context: ToEnvelope<EventHandlerInstance<Self>, M>,
    {
        let broker = crate::subscriber::SubscriberManager::<A>::from_registry();
        // let broker = T::get_broker();
        let recipient = ctx.address().recipient::<M>();
        broker.do_send(Subscribe(
            "$all".into(),
            recipient,
            std::any::type_name::<M>().to_string(),
        ));
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Subscribe<M: BrokerMsg>(pub String, pub Recipient<M>, pub String);

pub struct EventHandlerInstance<A: Application, E: EventHandler> {
    _phantom: std::marker::PhantomData<A>,
    pub(crate) _handler: E,
    pub(crate) _subscribtion: Addr<Subscriber>,
    pub(crate) _name: String,
}

impl<A: Application, E: EventHandler, V: BrokerMsg> actix::Handler<V>
    for EventHandlerInstance<A, E>
{
    // type Result = actix::MessageResult<V>;
    type Result = ();

    fn handle(&mut self, _event: V, _ctx: &mut Self::Context) -> Self::Result {}
}

impl<A: Application, E: EventHandler> EventHandlerInstance<A, E> {
    pub fn from_builder(builder: EventHandlerBuilder<E>) -> Addr<Self> {
        Self::create(move |ctx| {
            trace!("Register a new EventHandler instance with {}", builder.name);
            let ctx_address = ctx.address();

            EventHandlerInstance {
                _phantom: std::marker::PhantomData,
                _subscribtion: Subscriber::new(ctx_address.recipient(), "$all"),
                _handler: builder.handler,
                _name: builder.name,
            }
        })
    }
}
impl<A: Application, E: EventHandler> actix::Actor for EventHandlerInstance<A, E> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self._handler.started(ctx);
        // self.subscribe_sync::<BrokerType, EventNotification>(ctx);
    }
}

// impl<E: EventHandler> actix::Handler<EventNotification> for EventHandlerInstance<E> {
//     type Result = ();
//     fn handle(&mut self, _event: EventNotification, _: &mut Self::Context) -> Self::Result {
//         trace!("Received event notif on {}", self.name);
//     }
// }
impl<A: Application, E: EventHandler> actix::Handler<EventEnvelope> for EventHandlerInstance<A, E> {
    type Result = actix::ResponseActFuture<Self, Result<usize, ()>>;
    fn handle(&mut self, _event: EventEnvelope, _: &mut Self::Context) -> Self::Result {
        Box::pin(async { Ok(1) }.into_actor(self))
    }
}
