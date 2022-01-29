use crate::{
    command::Command, command::CommandMetadatas, message::Dispatch, Application,
    CommandExecutorError,
};
use actix::{ResponseFuture, SystemService};
use futures::TryFutureExt;
use tracing::trace;

#[doc(hidden)]
#[derive(Default)]
pub struct Router<A: Application> {
    pub(crate) _app: std::marker::PhantomData<A>,
    pub(crate) _before_dispatch: Vec<String>,
}

impl<A: Application> ::actix::Actor for Router<A> {
    type Context = ::actix::Context<Self>;
}

impl<A: Application> ::actix::registry::SystemService for Router<A> {}
impl<A: Application> ::actix::Supervised for Router<A> {}

impl<A: Application, T: Command> ::actix::Handler<Dispatch<T, A>> for Router<A>
where
    <T as Command>::ExecutorRegistry: actix::Handler<Dispatch<T, A>>,
{
    type Result = ResponseFuture<Result<Vec<T::Event>, CommandExecutorError>>;

    #[tracing::instrument(name = "Router", skip(self, _ctx, msg), fields(correlation_id = %msg.metadatas.correlation_id))]
    fn handle(&mut self, msg: Dispatch<T, A>, _ctx: &mut Self::Context) -> Self::Result {
        let to =
            <T::ExecutorRegistry as SystemService>::from_registry().recipient::<Dispatch<T, A>>();
        trace!(
            to = ::std::any::type_name::<T::ExecutorRegistry>(),
            "Route command",
        );
        Box::pin(to.send(msg).map_ok_or_else(|e| Err(e.into()), |r| r))
    }
}

impl<A: Application> Router<A> {
    #[tracing::instrument(name = "Dispatcher", skip(cmd, metadatas), fields(correlation_id = %metadatas.correlation_id))]
    pub async fn dispatch<C: Command>(
        cmd: C,
        metadatas: CommandMetadatas,
    ) -> Result<Vec<C::Event>, CommandExecutorError>
    where
        <C as Command>::ExecutorRegistry: actix::Handler<Dispatch<C, A>>,
    {
        trace!(
            executor = ::std::any::type_name::<C::Executor>(),
            "Sending {} to Router",
            ::std::any::type_name::<C>()
        );

        Self::from_registry()
            .send(Dispatch::<C, A> {
                metadatas,
                storage: std::marker::PhantomData,
                command: cmd,
            })
            .await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::aggregates::support::{InvalidCommand, MyApplication};
    use uuid::Uuid;

    #[actix::test]
    #[ignore]
    async fn can_route_a_command() {
        // TODO: Improve this tests with a particular APP
        let _ = Router::<MyApplication>::dispatch(
            InvalidCommand(Uuid::new_v4()),
            CommandMetadatas::default(),
        )
        .await;
    }
}
