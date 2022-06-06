use super::EventNotification;
use crate::{Application, SubscriberManager};
use actix::ActorContext;
use actix::{Actor, Addr, AsyncContext, Context};
use sqlx::postgres::PgNotification;
use std::convert::TryFrom;
use std::marker::PhantomData;
use tracing::trace;

pub struct Listener<A: Application> {
    _phantom: std::marker::PhantomData<A>,
    pub manager: Addr<SubscriberManager<A>>,
    pub listening: String,
}

impl<A: Application> actix::Actor for Listener<A> {
    type Context = Context<Self>;

    #[tracing::instrument(name = "Listener", skip(self, _ctx), fields(app = %A::get_name()))]
    fn started(&mut self, _ctx: &mut Self::Context) {
        trace!("Created a Listener instance");
    }
}

impl<A: Application> Listener<A> {
    #[tracing::instrument(name = "Listener", skip(url, manager), fields(app = %A::get_name()))]
    pub async fn setup(url: String, manager: Addr<SubscriberManager<A>>) -> Result<Addr<Self>, ()> {
        let mut listener = sqlx::postgres::PgListener::connect(&url).await.unwrap();
        listener.listen("events").await.unwrap();
        trace!("Starting listener with {}", url);

        Ok(Listener::create(move |ctx| {
            ctx.add_stream(listener.into_stream());

            Listener {
                _phantom: PhantomData,
                manager,
                listening: url,
            }
        }))
    }
}

impl<A: Application> actix::StreamHandler<Result<PgNotification, sqlx::Error>> for Listener<A> {
    fn handle(&mut self, item: Result<PgNotification, sqlx::Error>, _ctx: &mut Self::Context) {
        if let Ok(m) = item {
            if let Ok(event) = EventNotification::try_from(m.payload()) {
                self.manager.do_send(event);
            }
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        ctx.stop();
    }
}
