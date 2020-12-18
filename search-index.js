var searchIndex = JSON.parse('{\
"chekov":{"doc":"Chekov is a CQRS/ES Framework built on top of Actix actor…","i":[[3,"ApplicationBuilder","chekov","Struct to configure and launch an `Application` instance",null,null],[3,"SubscriberManager","","Deal with Application subscriptions",null,null],[0,"aggregate","","Aggregates produce events based on commands",null,null],[3,"AggregateInstance","chekov::aggregate","Deals with the lifetime of a particular aggregate",null,null],[8,"Aggregate","","Define an Aggregate",null,null],[10,"identity","","Define the identity of this kind of Aggregate.",0,[[]]],[0,"application","chekov","Struct and Trait correlated to Application",null,null],[3,"ApplicationBuilder","chekov::application","Struct to configure and launch an `Application` instance",null,null],[3,"DefaultEventResolver","","",null,null],[12,"mapping","","",1,null],[11,"event_handler","chekov","Adds an EventHandler to the application",2,[[["eventhandler",8]]]],[11,"with_storage_config","","Adds a StorageConfig use later to create the event_store…",2,[[["storageconfig",8]]]],[11,"storage","","Adds a preconfigured Backend as Storage. The storage isn\'t…",2,[[["future",8]]]],[11,"event_resolver","","Adds an EventResolver instance for this application",2,[[["eventresolver",8]]]],[11,"launch","","Launch the application",2,[[]]],[8,"Application","chekov::application","Application are high order logical seperator.",null,null],[16,"Storage","","The type of storage used by the application",3,null],[16,"EventResolver","","The type of the event_resolver, see EventResolver…",3,null],[11,"with_default","","Used to initiate the launch of the application",3,[[],["applicationbuilder",3]]],[11,"get_name","","Returns the logical name of the application Mostly used…",3,[[]]],[0,"event","chekov","Struct and Trait correlated to Event",null,null],[3,"EventHandlerInstance","chekov::event","Deals with the lifetime of a particular EventHandler",null,null],[11,"from_builder","","",4,[[["eventhandlerbuilder",3]],["addr",3]]],[8,"EventHandler","","Define a struct as an EventHandler",null,null],[11,"builder","","",5,[[],["eventhandlerbuilder",3]]],[11,"started","","",5,[[["context",3]]]],[11,"listen","","",5,[[["context",3]]]],[8,"Handler","","Receive an immutable event to handle",null,null],[10,"handle","","",6,[[],[["box",3],["pin",3]]]],[8,"Event","","Define an Event which can be produced and consumed",null,null],[11,"lazy_deserialize","","",7,[[],[["box",3],["fn",8]]]],[11,"register","","",7,[[]]],[8,"EventApplier","","Define an event applier",null,null],[10,"apply","","",8,[[],[["applyerror",4],["result",4]]]],[0,"prelude","chekov","",null,null],[3,"AggregateInstance","chekov::prelude","Deals with the lifetime of a particular aggregate",null,null],[3,"CommandMetadatas","","",null,null],[12,"command_id","","",9,null],[12,"correlation_id","","",9,null],[12,"causation_id","","",9,null],[12,"consistency","","",9,null],[3,"EventMetadatas","","",null,null],[12,"correlation_id","","",10,null],[12,"causation_id","","",10,null],[12,"stream_name","","",10,null],[4,"ApplyError","","Error returns by a failling EventApplier",null,null],[13,"Any","","",11,null],[4,"CommandExecutorError","","Error returns by a CommandExecutor",null,null],[13,"Any","","",12,null],[8,"Command","","Define a Command which can be dispatch",null,null],[16,"Event","","The Event that can be generated for this command",13,null],[16,"Executor","","The Executor that will execute the command and produce the…",13,null],[16,"ExecutorRegistry","","The registry where the command will be dispatched",13,null],[11,"get_correlation_id","","Returns the correlation id of the command",13,[[],["uuid",3]]],[11,"get_causation_id","","Returns the optional causation id of the command",13,[[],[["option",4],["uuid",3]]]],[10,"identifier","","Returns the identifier for this command.",13,[[],["string",3]]],[8,"CommandExecutor","","Receives a command and an immutable State and optionally…",null,null],[10,"execute","","",14,[[],[["result",4],["commandexecutorerror",4],["vec",3]]]],[8,"EventHandler","","Define a struct as an EventHandler",null,null],[11,"builder","chekov::event","",5,[[],["eventhandlerbuilder",3]]],[11,"started","","",5,[[["context",3]]]],[11,"listen","","",5,[[["context",3]]]],[8,"Aggregate","chekov","Define an Aggregate",null,null],[10,"identity","","Define the identity of this kind of Aggregate.",0,[[]]],[8,"Application","","Application are high order logical seperator.",null,null],[16,"Storage","","The type of storage used by the application",3,null],[16,"EventResolver","","The type of the event_resolver, see EventResolver…",3,null],[11,"with_default","","Used to initiate the launch of the application",3,[[],["applicationbuilder",3]]],[11,"get_name","","Returns the logical name of the application Mostly used…",3,[[]]],[8,"Command","","Define a Command which can be dispatch",null,null],[16,"Event","","The Event that can be generated for this command",13,null],[16,"Executor","","The Executor that will execute the command and produce the…",13,null],[16,"ExecutorRegistry","","The registry where the command will be dispatched",13,null],[11,"get_correlation_id","chekov::prelude","Returns the correlation id of the command",13,[[],["uuid",3]]],[11,"get_causation_id","","Returns the optional causation id of the command",13,[[],[["option",4],["uuid",3]]]],[10,"identifier","chekov","Returns the identifier for this command.",13,[[],["string",3]]],[8,"Event","","Define an Event which can be produced and consumed",null,null],[11,"lazy_deserialize","","",7,[[],[["box",3],["fn",8]]]],[11,"register","","",7,[[]]],[8,"EventApplier","","Define an event applier",null,null],[10,"apply","","",8,[[],[["applyerror",4],["result",4]]]],[8,"EventHandler","","Define a struct as an EventHandler",null,null],[11,"builder","chekov::event","",5,[[],["eventhandlerbuilder",3]]],[11,"started","","",5,[[["context",3]]]],[11,"listen","","",5,[[["context",3]]]],[8,"EventResolver","chekov","",null,null],[10,"resolve","","",15,[[["addr",3],["subscribermanager",3],["recordedevent",3]]]],[11,"from","","",2,[[]]],[11,"into","","",2,[[]]],[11,"borrow","","",2,[[]]],[11,"borrow_mut","","",2,[[]]],[11,"try_from","","",2,[[],["result",4]]],[11,"try_into","","",2,[[],["result",4]]],[11,"type_id","","",2,[[],["typeid",3]]],[11,"vzip","","",2,[[]]],[11,"from","","",16,[[]]],[11,"into","","",16,[[]]],[11,"borrow","","",16,[[]]],[11,"borrow_mut","","",16,[[]]],[11,"try_from","","",16,[[],["result",4]]],[11,"try_into","","",16,[[],["result",4]]],[11,"type_id","","",16,[[],["typeid",3]]],[11,"vzip","","",16,[[]]],[11,"from","chekov::aggregate","",17,[[]]],[11,"into","","",17,[[]]],[11,"borrow","","",17,[[]]],[11,"borrow_mut","","",17,[[]]],[11,"try_from","","",17,[[],["result",4]]],[11,"try_into","","",17,[[],["result",4]]],[11,"type_id","","",17,[[],["typeid",3]]],[11,"vzip","","",17,[[]]],[11,"from","chekov::application","",1,[[]]],[11,"into","","",1,[[]]],[11,"borrow","","",1,[[]]],[11,"borrow_mut","","",1,[[]]],[11,"try_from","","",1,[[],["result",4]]],[11,"try_into","","",1,[[],["result",4]]],[11,"type_id","","",1,[[],["typeid",3]]],[11,"vzip","","",1,[[]]],[11,"from","chekov::prelude","",9,[[]]],[11,"into","","",9,[[]]],[11,"to_owned","","",9,[[]]],[11,"clone_into","","",9,[[]]],[11,"borrow","","",9,[[]]],[11,"borrow_mut","","",9,[[]]],[11,"try_from","","",9,[[],["result",4]]],[11,"try_into","","",9,[[],["result",4]]],[11,"type_id","","",9,[[],["typeid",3]]],[11,"vzip","","",9,[[]]],[11,"from","","",12,[[]]],[11,"into","","",12,[[]]],[11,"borrow","","",12,[[]]],[11,"borrow_mut","","",12,[[]]],[11,"try_from","","",12,[[],["result",4]]],[11,"try_into","","",12,[[],["result",4]]],[11,"type_id","","",12,[[],["typeid",3]]],[11,"vzip","","",12,[[]]],[11,"from","","",11,[[]]],[11,"into","","",11,[[]]],[11,"borrow","","",11,[[]]],[11,"borrow_mut","","",11,[[]]],[11,"try_from","","",11,[[],["result",4]]],[11,"try_into","","",11,[[],["result",4]]],[11,"type_id","","",11,[[],["typeid",3]]],[11,"vzip","","",11,[[]]],[11,"from","chekov::event","",4,[[]]],[11,"into","","",4,[[]]],[11,"borrow","","",4,[[]]],[11,"borrow_mut","","",4,[[]]],[11,"try_from","","",4,[[],["result",4]]],[11,"try_into","","",4,[[],["result",4]]],[11,"type_id","","",4,[[],["typeid",3]]],[11,"vzip","","",4,[[]]],[11,"from","chekov::prelude","",10,[[]]],[11,"into","","",10,[[]]],[11,"to_owned","","",10,[[]]],[11,"clone_into","","",10,[[]]],[11,"borrow","","",10,[[]]],[11,"borrow_mut","","",10,[[]]],[11,"try_from","","",10,[[],["result",4]]],[11,"try_into","","",10,[[],["result",4]]],[11,"type_id","","",10,[[],["typeid",3]]],[11,"vzip","","",10,[[]]],[11,"resolve","chekov::application","",1,[[["addr",3],["subscribermanager",3],["recordedevent",3]]]],[11,"from","chekov::prelude","",12,[[["mailboxerror",4]]]],[11,"clone","","",9,[[],["commandmetadatas",3]]],[11,"clone","","",10,[[],["eventmetadatas",3]]],[11,"default","chekov::aggregate","",17,[[],["aggregateinstance",3]]],[11,"default","chekov","",2,[[]]],[11,"default","chekov::prelude","",9,[[]]],[11,"default","chekov","",16,[[]]],[11,"fmt","chekov::prelude","",9,[[["formatter",3]],["result",6]]],[11,"fmt","","",12,[[["formatter",3]],["result",6]]],[11,"fmt","","",11,[[["formatter",3]],["result",6]]],[11,"fmt","","",10,[[["formatter",3]],["result",6]]],[11,"started","chekov::event","",4,[[]]],[11,"started","chekov","",16,[[]]],[11,"serialize","chekov::prelude","",12,[[],["result",4]]],[11,"get_correlation_id","","Returns the correlation id of the command",13,[[],["uuid",3]]],[11,"get_causation_id","","Returns the optional causation id of the command",13,[[],[["option",4],["uuid",3]]]],[11,"builder","chekov::event","",5,[[],["eventhandlerbuilder",3]]],[11,"started","","",5,[[["context",3]]]],[11,"listen","","",5,[[["context",3]]]]],"p":[[8,"Aggregate"],[3,"DefaultEventResolver"],[3,"ApplicationBuilder"],[8,"Application"],[3,"EventHandlerInstance"],[8,"EventHandler"],[8,"Handler"],[8,"Event"],[8,"EventApplier"],[3,"CommandMetadatas"],[3,"EventMetadatas"],[4,"ApplyError"],[4,"CommandExecutorError"],[8,"Command"],[8,"CommandExecutor"],[8,"EventResolver"],[3,"SubscriberManager"],[3,"AggregateInstance"]]},\
"chekov_macros":{"doc":"","i":[[24,"Command","chekov_macros","",null,null],[24,"Aggregate","","",null,null],[24,"Event","","",null,null],[14,"apply_event","","",null,null]],"p":[]},\
"event_store":{"doc":"The `event_store` crate","i":[[3,"EventStore","event_store","An `EventStore` that hold a storage connection",null,null],[3,"EventStoreBuilder","","Builder use to simplify the `EventStore` creation",null,null],[5,"append","","Create an `Appender` to append events",null,[[],["appender",3]]],[5,"read","","Create a `Reader` to read a stream",null,[[],["reader",3]]],[0,"prelude","","",null,null],[3,"StreamInfo","event_store::prelude","",null,null],[12,"correlation_id","","",0,null],[12,"stream_uuid","","",0,null],[3,"RecordedEvent","","A `RecordedEvent` represents an `Event` which have been…",null,null],[12,"stream_uuid","","The stream identifier for thie event",1,null],[12,"causation_id","","a `causation_id` defines who caused this event",1,null],[12,"correlation_id","","a `correlation_id` correlates multiple events",1,null],[12,"event_type","","Human readable event type",1,null],[12,"data","","Payload of this event",1,null],[3,"RecordedEvents","","",null,null],[3,"UnsavedEvent","","An `UnsavedEvent` is created from a type that implement…",null,null],[3,"Appender","","An Appender defines the parameters of an append action",null,null],[3,"InMemoryBackend","","",null,null],[3,"PostgresBackend","","",null,null],[3,"Reader","","",null,null],[3,"Stream","","A `Stream` represents an `Event` stream",null,null],[4,"EventStoreError","","",null,null],[13,"Any","","",2,null],[13,"Storage","","",2,null],[4,"ExpectedVersion","","The `ExpectedVersion` used to define optimistic concurrency",null,null],[13,"AnyVersion","","Define that we expect a stream in any version",3,null],[13,"NoStream","","Define that we expect a non existing stream",3,null],[13,"StreamExists","","Define that we expect an existing stream",3,null],[13,"Version","","Define that we expect a stream in a particular version",3,null],[4,"ReadVersion","","",null,null],[13,"Origin","","",4,null],[13,"Version","","",4,null],[4,"StorageError","","",null,null],[13,"StreamDoesntExists","","",5,null],[13,"StreamAlreadyExists","","",5,null],[13,"Unknown","","",5,null],[8,"Event","","Represent event that can be handled by an `EventStore`",null,null],[10,"event_type","","Return a static str which define the event type",6,[[]]],[10,"all_event_types","","",6,[[],["vec",3]]],[11,"event_type_from_str","","",6,[[]]],[8,"Storage","","A `Storage` is responsible for storing and managing…",null,null],[10,"storage_name","","",7,[[]]],[10,"create_stream","","Create a new stream with an identifier",7,[[["stream",3],["uuid",3]],[["pin",3],["box",3]]]],[10,"delete_stream","","Delete a stream from the `Backend`",7,[[["stream",3],["uuid",3]],[["pin",3],["box",3]]]],[10,"append_to_stream","","",7,[[["uuid",3]],[["box",3],["pin",3]]]],[10,"read_stream","","",7,[[["string",3],["uuid",3]],[["pin",3],["box",3]]]],[10,"read_stream_info","","",7,[[["string",3],["uuid",3]],[["pin",3],["box",3]]]],[8,"Event","event_store","Represent event that can be handled by an `EventStore`",null,null],[10,"event_type","","Return a static str which define the event type",6,[[]]],[10,"all_event_types","","",6,[[],["vec",3]]],[11,"event_type_from_str","event_store::prelude","",6,[[]]],[11,"builder","event_store","",8,[[],["eventstorebuilder",3]]],[11,"storage","","Define which storage will be used by this building…",9,[[]]],[11,"build","","Try to build the previously configured `EventStore`",9,[[]]],[11,"from","","",8,[[]]],[11,"into","","",8,[[]]],[11,"to_owned","","",8,[[]]],[11,"clone_into","","",8,[[]]],[11,"borrow","","",8,[[]]],[11,"borrow_mut","","",8,[[]]],[11,"try_from","","",8,[[],["result",4]]],[11,"try_into","","",8,[[],["result",4]]],[11,"type_id","","",8,[[],["typeid",3]]],[11,"vzip","","",8,[[]]],[11,"from","","",9,[[]]],[11,"into","","",9,[[]]],[11,"borrow","","",9,[[]]],[11,"borrow_mut","","",9,[[]]],[11,"try_from","","",9,[[],["result",4]]],[11,"try_into","","",9,[[],["result",4]]],[11,"type_id","","",9,[[],["typeid",3]]],[11,"vzip","","",9,[[]]],[11,"from","event_store::prelude","",0,[[]]],[11,"into","","",0,[[]]],[11,"borrow","","",0,[[]]],[11,"borrow_mut","","",0,[[]]],[11,"try_from","","",0,[[],["result",4]]],[11,"try_into","","",0,[[],["result",4]]],[11,"type_id","","",0,[[],["typeid",3]]],[11,"vzip","","",0,[[]]],[11,"from","","",2,[[]]],[11,"into","","",2,[[]]],[11,"to_string","","",2,[[],["string",3]]],[11,"borrow","","",2,[[]]],[11,"borrow_mut","","",2,[[]]],[11,"try_from","","",2,[[],["result",4]]],[11,"try_into","","",2,[[],["result",4]]],[11,"type_id","","",2,[[],["typeid",3]]],[11,"vzip","","",2,[[]]],[11,"from","","",1,[[]]],[11,"into","","",1,[[]]],[11,"to_owned","","",1,[[]]],[11,"clone_into","","",1,[[]]],[11,"borrow","","",1,[[]]],[11,"borrow_mut","","",1,[[]]],[11,"try_from","","",1,[[],["result",4]]],[11,"try_into","","",1,[[],["result",4]]],[11,"type_id","","",1,[[],["typeid",3]]],[11,"vzip","","",1,[[]]],[11,"from","","",10,[[]]],[11,"into","","",10,[[]]],[11,"to_owned","","",10,[[]]],[11,"clone_into","","",10,[[]]],[11,"borrow","","",10,[[]]],[11,"borrow_mut","","",10,[[]]],[11,"try_from","","",10,[[],["result",4]]],[11,"try_into","","",10,[[],["result",4]]],[11,"type_id","","",10,[[],["typeid",3]]],[11,"vzip","","",10,[[]]],[11,"from","","",11,[[]]],[11,"into","","",11,[[]]],[11,"to_owned","","",11,[[]]],[11,"clone_into","","",11,[[]]],[11,"borrow","","",11,[[]]],[11,"borrow_mut","","",11,[[]]],[11,"try_from","","",11,[[],["result",4]]],[11,"try_into","","",11,[[],["result",4]]],[11,"type_id","","",11,[[],["typeid",3]]],[11,"vzip","","",11,[[]]],[11,"from","","",3,[[]]],[11,"into","","",3,[[]]],[11,"borrow","","",3,[[]]],[11,"borrow_mut","","",3,[[]]],[11,"try_from","","",3,[[],["result",4]]],[11,"try_into","","",3,[[],["result",4]]],[11,"type_id","","",3,[[],["typeid",3]]],[11,"vzip","","",3,[[]]],[11,"from","","",4,[[]]],[11,"into","","",4,[[]]],[11,"borrow","","",4,[[]]],[11,"borrow_mut","","",4,[[]]],[11,"try_from","","",4,[[],["result",4]]],[11,"try_into","","",4,[[],["result",4]]],[11,"type_id","","",4,[[],["typeid",3]]],[11,"vzip","","",4,[[]]],[11,"from","","",5,[[]]],[11,"into","","",5,[[]]],[11,"borrow","","",5,[[]]],[11,"borrow_mut","","",5,[[]]],[11,"try_from","","",5,[[],["result",4]]],[11,"try_into","","",5,[[],["result",4]]],[11,"type_id","","",5,[[],["typeid",3]]],[11,"vzip","","",5,[[]]],[11,"from","","",12,[[]]],[11,"into","","",12,[[]]],[11,"borrow","","",12,[[]]],[11,"borrow_mut","","",12,[[]]],[11,"try_from","","",12,[[],["result",4]]],[11,"try_into","","",12,[[],["result",4]]],[11,"type_id","","",12,[[],["typeid",3]]],[11,"vzip","","",12,[[]]],[11,"from","","",13,[[]]],[11,"into","","",13,[[]]],[11,"borrow","","",13,[[]]],[11,"borrow_mut","","",13,[[]]],[11,"try_from","","",13,[[],["result",4]]],[11,"try_into","","",13,[[],["result",4]]],[11,"type_id","","",13,[[],["typeid",3]]],[11,"vzip","","",13,[[]]],[11,"from","","",14,[[]]],[11,"into","","",14,[[]]],[11,"borrow","","",14,[[]]],[11,"borrow_mut","","",14,[[]]],[11,"try_from","","",14,[[],["result",4]]],[11,"try_into","","",14,[[],["result",4]]],[11,"type_id","","",14,[[],["typeid",3]]],[11,"vzip","","",14,[[]]],[11,"from","","",15,[[]]],[11,"into","","",15,[[]]],[11,"borrow","","",15,[[]]],[11,"borrow_mut","","",15,[[]]],[11,"try_from","","",15,[[],["result",4]]],[11,"try_into","","",15,[[],["result",4]]],[11,"type_id","","",15,[[],["typeid",3]]],[11,"vzip","","",15,[[]]],[11,"from","","",16,[[]]],[11,"into","","",16,[[]]],[11,"to_owned","","",16,[[]]],[11,"clone_into","","",16,[[]]],[11,"borrow","","",16,[[]]],[11,"borrow_mut","","",16,[[]]],[11,"try_from","","",16,[[],["result",4]]],[11,"try_into","","",16,[[],["result",4]]],[11,"type_id","","",16,[[],["typeid",3]]],[11,"vzip","","",16,[[]]],[11,"storage_name","","",13,[[]]],[11,"create_stream","","",13,[[["stream",3],["uuid",3]],[["pin",3],["box",3]]]],[11,"delete_stream","","",13,[[["stream",3],["uuid",3]],[["pin",3],["box",3]]]],[11,"read_stream","","",13,[[["string",3],["uuid",3]],[["pin",3],["box",3]]]],[11,"append_to_stream","","",13,[[["uuid",3]],[["box",3],["pin",3]]]],[11,"read_stream_info","","",13,[[["string",3],["uuid",3]],[["pin",3],["box",3]]]],[11,"storage_name","","",14,[[]]],[11,"create_stream","","",14,[[["stream",3],["uuid",3]],[["pin",3],["box",3]]]],[11,"delete_stream","","",14,[[["stream",3],["uuid",3]],[["pin",3],["box",3]]]],[11,"read_stream","","",14,[[["string",3],["uuid",3]],[["pin",3],["box",3]]]],[11,"append_to_stream","","",14,[[["uuid",3]],[["box",3],["pin",3]]]],[11,"read_stream_info","","",14,[[["string",3],["uuid",3]],[["pin",3],["box",3]]]],[11,"from","","",2,[[["mailboxerror",4]]]],[11,"from","","",5,[[["error",4]]]],[11,"clone","","",1,[[],["recordedevent",3]]],[11,"clone","","",10,[[],["recordedevents",3]]],[11,"clone","","",11,[[],["unsavedevent",3]]],[11,"clone","","",16,[[],["stream",3]]],[11,"clone","event_store","",8,[[],["eventstore",3]]],[11,"default","event_store::prelude","",12,[[]]],[11,"default","","",13,[[],["inmemorybackend",3]]],[11,"default","","",14,[[]]],[11,"default","","",15,[[]]],[11,"default","event_store","",8,[[]]],[11,"eq","event_store::prelude","",2,[[["eventstoreerror",4]]]],[11,"ne","","",2,[[["eventstoreerror",4]]]],[11,"eq","","",11,[[["unsavedevent",3]]]],[11,"ne","","",11,[[["unsavedevent",3]]]],[11,"eq","","",3,[[["expectedversion",4]]]],[11,"ne","","",3,[[["expectedversion",4]]]],[11,"eq","","",5,[[["storageerror",4]]]],[11,"eq","","",16,[[["stream",3]]]],[11,"ne","","",16,[[["stream",3]]]],[11,"fmt","","",2,[[["formatter",3]],["result",6]]],[11,"fmt","","",1,[[["formatter",3]],["result",6]]],[11,"fmt","","",10,[[["formatter",3]],["result",6]]],[11,"fmt","","",11,[[["formatter",3]],["result",6]]],[11,"fmt","","",3,[[["formatter",3]],["result",6]]],[11,"fmt","","",4,[[["formatter",3]],["result",6]]],[11,"fmt","","",13,[[["formatter",3]],["result",6]]],[11,"fmt","","",14,[[["formatter",3]],["result",6]]],[11,"fmt","","",5,[[["formatter",3]],["result",6]]],[11,"fmt","","",16,[[["formatter",3]],["result",6]]],[11,"fmt","event_store","",9,[[["formatter",3]],["result",6]]],[11,"fmt","event_store::prelude","",2,[[["formatter",3]],["result",6]]],[11,"from_str","","",16,[[],["result",4]]],[11,"handle","event_store","",8,[[["streaminfo",3]]]],[11,"from_row","event_store::prelude","",1,[[],["result",6]]],[11,"try_deserialize","","Errors`()` for now",1,[[],[["deserialize",8],["result",4],["deserialize",8],["event",8]]]],[11,"try_from","","ErrorsIf `serde` isn\'t able to serialize the `Event`",11,[[],[["parseeventerror",4],["result",4]]]],[11,"event_type_from_str","","",6,[[]]],[11,"verify","","",3,[[["stream",3]],["expectedversionresult",4]]],[11,"with_correlation_id","","",12,[[["uuid",3]]]],[11,"events","","Add a list of `Event`s to the `Appender`",12,[[],[["eventstoreerror",4],["result",4]]]],[11,"event","","Add a single `Event`s to the `Appender`",12,[[],[["eventstoreerror",4],["result",4]]]],[11,"to","","Define which stream we are appending to",12,[[["tostring",8]],[["eventstoreerror",4],["result",4]]]],[11,"expected_version","","Define the expected version of the stream we are appending…",12,[[["expectedversion",4]]]],[11,"execute","","Execute the `Appender` against a `Storage` and returns the…",12,[[["eventstore",3],["addr",3]]]],[11,"with_url","","ErrorsIn case of Postgres connection error",14,[[]]],[11,"with_correlation_id","","",15,[[["uuid",3]]]],[11,"stream","","Define which stream we are reading from",15,[[["into",8],["string",3]],[["eventstoreerror",4],["result",4]]]],[11,"from","","",15,[[["readversion",4]]]],[11,"limit","","",15,[[]]],[11,"execute_async","","",15,[[["eventstore",3],["addr",3]]]],[11,"stream_uuid","","",16,[[]]]],"p":[[3,"StreamInfo"],[3,"RecordedEvent"],[4,"EventStoreError"],[4,"ExpectedVersion"],[4,"ReadVersion"],[4,"StorageError"],[8,"Event"],[8,"Storage"],[3,"EventStore"],[3,"EventStoreBuilder"],[3,"RecordedEvents"],[3,"UnsavedEvent"],[3,"Appender"],[3,"InMemoryBackend"],[3,"PostgresBackend"],[3,"Reader"],[3,"Stream"]]}\
}');
addSearchOptions(searchIndex);initSearch(searchIndex);