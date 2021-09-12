use super::support::*;
use crate::aggregate::AggregateInstance;
use crate::assert_aggregate_version;
use crate::message::Dispatch;
use crate::prelude::*;
use actix::Addr;
use demonstrate::demonstrate;
use std::marker::PhantomData;
use uuid::Uuid;

demonstrate! {
    describe "aggregate" {
        use super::*;


        #[actix::test]
        async it "should be able to start" -> Result<(), Box<dyn std::error::Error>> {
            let instance = start_aggregate().await;

            assert_aggregate_version!(instance, 0);

            let _ = instance.send(Dispatch::<_, MyApplication>{
                metadatas: CommandMetadatas::default(),
                storage: PhantomData,
                command: AppendItem(1)
            }).await;

            assert_aggregate_version!(instance, 1);

            Ok(())
        }
    }
}

async fn start_aggregate() -> Addr<AggregateInstance<ExampleAggregate>> {
    MyApplication::with_default()
        .storage(event_store::prelude::InMemoryBackend::initiate())
        .launch()
        .await;

    let identity = "my_aggregate";
    let correlation_id = Uuid::new_v4();

    AggregateInstance::<ExampleAggregate>::new::<MyApplication>(
        identity.to_string(),
        correlation_id,
    )
    .await
    .unwrap()
}
