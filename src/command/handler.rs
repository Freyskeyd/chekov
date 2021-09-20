use actix::Context;
use futures::future::BoxFuture;

use crate::aggregate::StaticState;
use crate::command::Handler;
use crate::prelude::CommandExecutor;
use crate::prelude::CommandExecutorError;
use crate::Command;

use super::CommandHandler;

pub(crate) mod instance;
mod registry;

#[derive(Default)]
pub struct NoHandler {}

impl CommandHandler for NoHandler {}

impl<C: Command, A: CommandExecutor<C> + Sync> Handler<C, A> for NoHandler {
    fn handle(
        &mut self,
        _: C,
        _: StaticState<A>,
    ) -> BoxFuture<'static, Result<Vec<C::Event>, CommandExecutorError>> {
        unimplemented!()
    }
}

impl actix::SystemService for NoHandler {}
impl actix::Supervised for NoHandler {}
impl actix::Actor for NoHandler {
    type Context = Context<Self>;
}
