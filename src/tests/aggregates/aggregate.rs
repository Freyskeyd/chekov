use super::support::*;
use crate::aggregate::AggregateInstance;
use crate::assert_aggregate_version;
use crate::event_store::EventStore;
use crate::message::Dispatch;
use crate::prelude::*;
use actix::Addr;
use demonstrate::demonstrate;
use event_store::prelude::Appender;
use std::marker::PhantomData;
use uuid::Uuid;

demonstrate! {
    describe "aggregate" {
        use super::*;


        #[actix::test]
        async it "should be able to start" -> Result<(), Box<dyn std::error::Error>> {
            let identifier = Uuid::new_v4();
            start_application().await;
            let instance = start_aggregate(identifier).await;

            assert_aggregate_version!(instance, 0);

            let _ = instance.send(Dispatch::<_, MyApplication>{
                metadatas: CommandMetadatas::default(),
                storage: PhantomData,
                command: AppendItem(1, identifier)
            }).await;

            assert_aggregate_version!(instance, 1);

            Ok(())
        }

        #[actix::test]
        async it "Should rebuild his state from previously append events" -> Result<(), Box<dyn std::error::Error>> {
            start_application().await;

            let identifier = Uuid::new_v4();
            let _ = EventStore::<MyApplication>::with_appender(
                Appender::default()
                .event(&ItemAppended(1))
                .unwrap()
                .to(&identifier)
                .unwrap(),
            ).await;

            let instance = start_aggregate(identifier).await;

            assert_aggregate_version!(instance, 1);

            let result = instance.send(Dispatch::<_, MyApplication>{
                metadatas: CommandMetadatas::default(),
                storage: PhantomData,
                command: AppendItem(1, identifier)
            }).await;

            assert!(result.is_ok());

            assert_aggregate_version!(instance, 2);

            Ok(())
        }

    }
}

async fn start_context(identity: Uuid) -> Addr<AggregateInstance<ExampleAggregate>> {
    start_application().await;
    start_aggregate(identity).await
}

async fn start_application() {
    MyApplication::with_default()
        .storage(event_store::prelude::InMemoryBackend::initiate())
        .launch()
        .await;
}

async fn start_aggregate(identity: Uuid) -> Addr<AggregateInstance<ExampleAggregate>> {
    let correlation_id = Uuid::new_v4();

    AggregateInstance::<ExampleAggregate>::new::<MyApplication>(
        identity.to_string(),
        correlation_id,
    )
    .await
    .unwrap()
}
