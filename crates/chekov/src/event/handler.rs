use std::sync::Arc;

use crate::message::ResolveAndApplyMany;
use crate::Application;
use crate::{error::HandleError, event_store::EventStore};
use actix::prelude::*;
use event_store::prelude::{StartFrom, SubscriptionNotification};
use tracing::trace;

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

/// Define a struct as an EventHandler
#[async_trait::async_trait]
pub trait EventHandler: Clone + Sized + std::marker::Unpin + 'static {
    fn builder(self) -> EventHandlerBuilder<Self> {
        EventHandlerBuilder::new(self)
    }

    async fn handle_recorded_event(
        state: &mut Self,
        event: event_store::prelude::RecordedEvent,
    ) -> Result<(), HandleError>;

    fn listen<A: Application>(&self, _ctx: &mut actix::Context<EventHandlerInstance<A, Self>>) {
        // let broker = crate::subscriber::SubscriberManager::<A>::from_registry();
        // let recipient = ctx.address().recipient::<ResolveAndApplyMany>();
        // let recipient_event = ctx.address().recipient::<SubscriptionNotification>();
        // broker.do_send(Subscribe("$all".into(), recipient, recipient_event));
    }

    fn started<A: Application>(&mut self, _ctx: &mut actix::Context<EventHandlerInstance<A, Self>>)
    where
        Self: EventHandler,
    {
        // self.listen(ctx);
    }
}

#[doc(hidden)]
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Subscribe {
    pub stream: String,
    pub resolver: Recipient<ResolveAndApplyMany>,
    pub recipient: Recipient<SubscriptionNotification>,
    pub transient: bool,
}

/// Deals with the lifetime of a particular EventHandler
pub struct EventHandlerInstance<A: Application, E: EventHandler> {
    _phantom: std::marker::PhantomData<A>,
    pub(crate) handler: E,
    pub(crate) _name: String,
}

impl<A: Application, E: EventHandler> EventHandlerInstance<A, E> {
    #[tracing::instrument(name = "EventHandlerInstance", skip(builder))]
    pub fn from_builder(builder: EventHandlerBuilder<E>) -> Addr<Self> {
        Self::create(move |_ctx| {
            trace!("Register a new EventHandler instance with {}", builder.name);

            EventHandlerInstance {
                _phantom: std::marker::PhantomData,
                handler: builder.handler,
                _name: builder.name,
            }
        })
    }
}

impl<A: Application, E: EventHandler> actix::Actor for EventHandlerInstance<A, E> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let opts = event_store::prelude::SubscriptionOptions {
            stream_uuid: "$all".to_owned(),
            subscription_name: self._name.to_owned(),
            start_from: StartFrom::Origin,
            transient: false,
        };

        let addr = ctx.address().recipient();
        let fut = async move {
            let storage = EventStore::<A>::get_addr().await.unwrap();
            // TODO: Handle error in case of subscription failure
            let _ = event_store::prelude::Subscriptions::<A::Storage>::subscribe_to_stream(
                addr, opts, storage,
            )
            .await;
        };

        ctx.spawn(fut.into_actor(self));

        EventHandler::started(&mut self.handler, ctx);
    }
}

impl<A: Application, E: EventHandler> ::actix::Handler<SubscriptionNotification>
    for EventHandlerInstance<A, E>
{
    type Result = ResponseFuture<Result<(), ()>>;

    #[tracing::instrument(name = "EventHandlerInstance", skip(self, msg, _ctx))]
    fn handle(&mut self, msg: SubscriptionNotification, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SubscriptionNotification::OwnedEvents(_events) => {
                // let handler = self.handler.clone();
                // let events = events;

                Box::pin(async move {
                    // for event in events {
                    //     // TODO: Deal with handle error
                    //     EventHandler::handle_recorded_event(&mut handler, Arc::into_raw(event).clone())
                    //         .await
                    //         .map_err(|_| ())?;
                    // }
                    Ok(())
                })
            }

            SubscriptionNotification::PubSubEvents(stream, events) => {
                let mut handler = self.handler.clone();

                Box::pin(async move {
                    for event in events {
                        // TODO: Remove clonning to prevent data duplication
                        EventHandler::handle_recorded_event(&mut handler, event.as_ref().clone())
                            .await;
                    }
                    Ok(())
                })
            }

            SubscriptionNotification::Events(events) => {
                let mut handler = self.handler.clone();
                let events = events;
                Box::pin(async move {
                    for event in events.iter() {
                        // TODO: Deal with handle error
                        // TODO: Remove clonning to prevent data duplication
                        EventHandler::handle_recorded_event(&mut handler, event.as_ref().clone())
                            .await
                            .map_err(|_| ())?;
                    }
                    Ok(())
                })
            }
            SubscriptionNotification::Subscribed => Box::pin(async { Ok(()) }),
        }
    }
}
