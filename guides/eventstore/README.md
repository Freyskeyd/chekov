# EventStore

The `EventStore` will allow you to deal with every aspects of the event sourcing part of Chekov.


## Appending an event


Events are appended by using the fluent API exposed at the root level of the event_store crate:

```rust
use event_store::prelude::*;

let stream_uuid = Uuid::new_v4().to_string();
let my_event = MyEvent { account_id: Uuid::new_v4() };

event_store::append()
  .event(&my_event)?
  .to(&stream_uuid)
  .execute(&event_store)
  .await;
```

## Reading from stream


A `Stream` can be read with the fluent API exposed at the root level of the event_store crate:

```rust
use event_store::prelude::*;

let stream_uuid = Uuid::new_v4().to_string();

event_store::read()
  .stream(&stream_uuid)
  .from(ReadVersion::Origin)
  .limit(10)
  .execute(&event_store)
  .await;
```
