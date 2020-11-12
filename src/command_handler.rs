use crate::Command;
use crate::CommandExecutorError;

pub trait CommandHandler<C: Command + ?Sized> {
    fn execute(command: C, executor: &C::Executor) -> Result<Vec<C::Event>, CommandExecutorError>;
}
