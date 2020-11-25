use super::*;

#[async_trait::async_trait]
pub trait Command: std::fmt::Debug + Send + 'static {
    type Event: Event;
    type Executor: CommandExecutor<Self> + EventApplier<Self::Event>;
    type ExecutorRegistry: ArbiterService;

    fn identifier(&self) -> String;
    async fn dispatch(&self) -> Result<(), ()>;
}

#[async_trait::async_trait]
pub trait Dispatchable<C, A>
where
    C: Command,
    A: Application,
{
    async fn dispatch(&self, cmd: C) -> Result<Vec<C::Event>, CommandExecutorError>
    where
        <C as Command>::ExecutorRegistry: actix::Handler<Dispatch<C, A>>;
}
