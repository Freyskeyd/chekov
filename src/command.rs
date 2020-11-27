use super::*;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait Command: std::fmt::Debug + Send + 'static {
    type Event: Event;
    type Executor: CommandExecutor<Self> + EventApplier<Self::Event>;
    type ExecutorRegistry: ArbiterService;

    fn get_correlation_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    fn get_causation_id(&self) -> Option<Uuid> {
        Some(Uuid::new_v4())
    }

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

#[derive(Debug, Clone)]
pub enum Consistency {
    Strong,
    Eventual,
}

#[derive(Debug, Clone)]
pub struct CommandMetadatas {
    pub command_id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub consistency: Consistency,
}

impl std::default::Default for CommandMetadatas {
    fn default() -> Self {
        Self {
            command_id: uuid::Uuid::new_v4(),
            correlation_id: uuid::Uuid::new_v4(),
            causation_id: None,
            consistency: Consistency::Eventual,
        }
    }
}
