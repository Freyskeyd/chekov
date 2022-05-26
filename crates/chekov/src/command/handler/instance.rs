use std::time::Duration;

use crate::command::{CommandExecutor, Handler};
use crate::{
    command::CommandHandler, message::DispatchWithState, prelude::CommandExecutorError, Aggregate,
    Application, Command,
};
use actix::prelude::Handler as ActixHandler;
use actix::prelude::*;
use futures::{FutureExt, TryFutureExt};

type DispatchResult<A, E> = ResponseFuture<Result<(Vec<E>, A), CommandExecutorError>>;

#[derive(Default)]
pub struct CommandHandlerInstance<H: CommandHandler> {
    pub(crate) inner: H,
}

impl<H: CommandHandler> SystemService for CommandHandlerInstance<H> {}
impl<H: CommandHandler> Supervised for CommandHandlerInstance<H> {}
impl<H: CommandHandler> Actor for CommandHandlerInstance<H> {
    type Context = Context<Self>;
}

impl<A: Aggregate, C: Command, APP: Application, H: CommandHandler>
    ActixHandler<DispatchWithState<A, C, APP>> for CommandHandlerInstance<H>
where
    H: Handler<C, A>,
    A: CommandExecutor<C>,
{
    type Result = DispatchResult<A, C::Event>;
    fn handle(
        &mut self,
        cmd: DispatchWithState<A, C, APP>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let command = cmd.command;
        let state = cmd.state;
        let fut = self.inner.handle(command, state.clone());
        Box::pin(
            tokio::time::timeout(Duration::from_secs(5), fut)
                .map(|res| match res {
                    Ok(response) => response.map(|events| (events, state)),
                    Err(error) => {
                        tracing::error!("First : {:?}", error);
                        Err(error.into())
                    }
                })
                .map_err(|error| {
                    tracing::error!("{:?}", error);
                    error
                }),
        )
    }
}
