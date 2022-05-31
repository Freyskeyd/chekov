use std::marker::PhantomData;

use actix::Message;

use crate::{aggregate::StaticState, Aggregate};

pub struct CommandExecutionResult<E, A> {
    pub events: Vec<E>,
    pub new_version: i64,
    pub state: A,
}
