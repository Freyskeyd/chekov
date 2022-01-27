use std::time::Duration;

use crate::command::{CommandExecutor, Handler};
use crate::{
    command::CommandHandler, message::DispatchWithState, prelude::CommandExecutorError, Aggregate,
    Application, Command,
};
use actix::prelude::Handler as ActixHandler;
use actix::prelude::*;

type DispatchResult<H, A, E> = ResponseActFuture<H, Result<(Vec<E>, A), CommandExecutorError>>;

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
    type Result = DispatchResult<Self, A, C::Event>;
    fn handle(
        &mut self,
        cmd: DispatchWithState<A, C, APP>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let command = cmd.command;
        let state = cmd.state;
        let fut = self.inner.handle(command, state.clone());
        Box::pin(
            async move { fut.await }
                .into_actor(self)
                .timeout(Duration::from_secs(5))
                .map(|res, _, _| match res {
                    Ok(r) => r.map(|events| (events, state)),
                    Err(e) => {
                        tracing::error!("First : {:?}", e);
                        Err(CommandExecutorError::Any)
                    }
                })
                .map_err(|a, _, c| {
                    tracing::error!("{:?}", a);
                    tracing::error!("{:?}", c);
                    CommandExecutorError::Any
                }),
        )
    }
}
