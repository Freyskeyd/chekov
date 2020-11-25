use crate::{command::Command, message::Dispatch, Application, CommandExecutorError};
use actix::prelude::{ArbiterService, WrapFuture};

#[derive(Default)]
pub struct Router<A: Application> {
    pub(crate) _app: std::marker::PhantomData<A>,
    pub(crate) _before_dispatch: Vec<String>,
}

impl<A: Application> ::actix::Actor for Router<A> {
    type Context = ::actix::Context<Self>;
}

impl<A: Application> ::actix::registry::ArbiterService for Router<A> {}
impl<A: Application> ::actix::Supervised for Router<A> {}

impl<A: Application, T: Command> ::actix::Handler<Dispatch<T, A>> for Router<A>
where
    <T as Command>::ExecutorRegistry: actix::Handler<Dispatch<T, A>>,
{
    type Result = actix::ResponseActFuture<Self, Result<Vec<T::Event>, CommandExecutorError>>;

    fn handle(&mut self, msg: Dispatch<T, A>, _ctx: &mut Self::Context) -> Self::Result {
        let to =
            <T::ExecutorRegistry as ArbiterService>::from_registry().recipient::<Dispatch<T, A>>();
        Box::pin(async move { to.send(msg).await? }.into_actor(self))
    }
}

impl<A: Application> Router<A> {
    pub async fn dispatch<C: Command>(cmd: C) -> Result<Vec<C::Event>, CommandExecutorError>
    where
        <C as Command>::ExecutorRegistry: actix::Handler<Dispatch<C, A>>,
    {
        Self::from_registry()
            .send(Dispatch::<C, A> {
                storage: std::marker::PhantomData,
                command: cmd,
            })
            .await?
    }
}
