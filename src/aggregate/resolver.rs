use super::*;
use std::{any::TypeId, collections::BTreeMap};

pub trait EventResolverItem<A: Aggregate> {
    fn get_names(&self) -> &[&'static str];
}

pub type EventApplierFn<A> = fn(&mut A, RecordedEvent) -> Result<(), ApplyError>;

pub struct EventResolverRegistry<A: Aggregate> {
    pub names: BTreeMap<&'static str, TypeId>,
    pub appliers: BTreeMap<TypeId, EventApplierFn<A>>,
}

impl<A: Aggregate> EventResolverRegistry<A> {
    pub fn get_applier(&self, event_name: &str) -> Option<&EventApplierFn<A>> {
        let type_id = self.names.get(event_name)?;

        self.appliers.get(type_id)
    }
}
