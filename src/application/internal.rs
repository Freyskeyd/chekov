use std::marker::PhantomData;

use crate::Application;
use actix::prelude::*;

pub(crate) struct InternalApplication<A: Application> {
    pub(crate) _phantom: PhantomData<A>,
}

impl<A> std::default::Default for InternalApplication<A>
where
    A: Application,
{
    fn default() -> Self {
        unimplemented!()
    }
}

impl<A> actix::Actor for InternalApplication<A>
where
    A: Application,
{
    type Context = Context<Self>;
}

impl<A> actix::SystemService for InternalApplication<A> where A: Application {}
impl<A> actix::Supervised for InternalApplication<A> where A: Application {}
