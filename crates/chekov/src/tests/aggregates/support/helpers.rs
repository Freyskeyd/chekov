use actix::Addr;
use uuid::Uuid;

use crate::{
    aggregate::{AggregateInstance, AggregateInstanceRegistry},
    Application,
};

use super::{ExampleAggregate, MyApplication};

#[allow(dead_code)]
pub(crate) async fn start_context(identity: &Uuid) -> Addr<AggregateInstance<ExampleAggregate>> {
    start_application().await;
    start_aggregate(identity).await
}

pub(crate) async fn start_application() {
    MyApplication::with_default()
        .storage(event_store::storage::InMemoryStorage::initiate())
        .launch()
        .await;
}

pub(crate) async fn start_aggregate(identity: &Uuid) -> Addr<AggregateInstance<ExampleAggregate>> {
    let correlation_id = Uuid::new_v4();

    AggregateInstanceRegistry::<ExampleAggregate>::start_aggregate::<MyApplication>(
        identity.to_string(),
        Some(correlation_id),
    )
    .await
    .unwrap()
}
