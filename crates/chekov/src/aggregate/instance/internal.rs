pub struct CommandExecutionResult<E, A> {
    pub events: Vec<E>,
    pub new_version: u64,
    pub state: A,
}
