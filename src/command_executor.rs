use crate::Aggregate;
use crate::Command;
use crate::CommandExecutorError;

pub trait CommandExecutor<T: Command + ?Sized>: Aggregate {
    fn execute(cmd: T, state: &Self) -> Result<Vec<T::Event>, CommandExecutorError>;
}
