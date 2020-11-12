use crate::{command::Command, command::Dispatchable, message::Dispatch, CommandExecutorError};
use actix::prelude::{ArbiterService, WrapFuture};

pub struct Router<S: event_store::prelude::Storage> {
    pub(crate) _event_store: event_store::EventStore<S>,
    pub(crate) _before_dispatch: Vec<String>,
}

impl<S: event_store::prelude::Storage> std::default::Default for Router<S> {
    fn default() -> Self {
        unimplemented!()
    }
}
impl<S: event_store::prelude::Storage> ::actix::Actor for Router<S> {
    type Context = ::actix::Context<Self>;
}

impl<S: event_store::prelude::Storage> ::actix::registry::ArbiterService for Router<S> {}
impl<S: event_store::prelude::Storage> ::actix::Supervised for Router<S> {}

#[async_trait::async_trait]
impl<S: event_store::prelude::Storage, C: Command> Dispatchable<C, S> for Router<S> {
    async fn dispatch(&self, cmd: C) -> Result<Vec<C::Event>, CommandExecutorError>
    where
        <C as Command>::ExecutorRegistry: actix::Handler<Dispatch<C, S>>,
    {
        Self::from_registry()
            .send(Dispatch::<C, S> {
                storage: std::marker::PhantomData,
                // to: <C::ExecutorRegistry as ArbiterService>::from_registry()
                //     .recipient::<Dispatch<C, S>>(),
                command: cmd,
            })
            .await?
        // // Execute before_dispatch middleware
        // // Open aggregate
        // // Create ExecutionContext
        // // Execute after_dispatch middleware
    }
}
impl<S: event_store::prelude::Storage, T: Command> ::actix::Handler<Dispatch<T, S>> for Router<S>
where
    <T as Command>::ExecutorRegistry: actix::Handler<Dispatch<T, S>>,
{
    type Result = actix::ResponseActFuture<Self, Result<Vec<T::Event>, CommandExecutorError>>;

    fn handle(&mut self, msg: Dispatch<T, S>, _ctx: &mut Self::Context) -> Self::Result {
        // let to = msg.to.clone();

        let to =
            <T::ExecutorRegistry as ArbiterService>::from_registry().recipient::<Dispatch<T, S>>();
        Box::pin(async move { to.send(msg).await? }.into_actor(self))
    }
}
