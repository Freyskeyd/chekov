var searchIndex = JSON.parse('{\
"chekov":{"doc":"Chekov is a CQRS/ES Framework built on top of Actix actor …","t":[8,8,3,8,8,16,8,8,16,16,16,3,0,0,23,10,10,11,11,11,0,23,11,11,10,10,10,11,11,0,11,11,11,11,11,11,8,10,10,8,3,16,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,8,8,8,8,10,11,11,11,11,10,10,11,11,11,11,11,11,11,11,11,24,24,24,24,23,23,3,13,13,4,8,8,4,3,16,8,3,16,16,11,11,11,11,11,11,11,11,11,11,12,12,11,11,11,11,12,12,12,12,11,11,11,10,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,10,10,11,11,11,11,11,11,11,11,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11],"n":["Aggregate","Application","ApplicationBuilder","Command","Event","Event","EventApplier","EventHandler","Executor","ExecutorRegistry","Storage","SubscriberManager","aggregate","application","applier","apply","apply_recorded_event","borrow","borrow_mut","default","event","event_handler","from","get_name","handle_recorded_event","identifier","identity","into","into_envelope","prelude","started","try_from","try_into","type_id","vzip","with_default","Aggregate","apply_recorded_event","identity","Application","ApplicationBuilder","Storage","borrow","borrow_mut","default","event_handler","from","get_name","into","launch","storage","try_from","try_into","type_id","vzip","with_default","with_storage_config","Event","EventApplier","EventHandler","Handler","apply","builder","builder","builder","builder","handle","handle_recorded_event","into_envelope","listen","listen","listen","listen","started","started","started","started","Aggregate","Command","Event","EventHandler","applier","event_handler","AggregateInstance","Any","Any","ApplyError","Command","CommandExecutor","CommandExecutorError","CommandMetadatas","Event","EventHandler","EventMetadatas","Executor","ExecutorRegistry","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","causation_id","causation_id","clone","clone","clone_into","clone_into","command_id","consistency","correlation_id","correlation_id","default","default","erased_serialize","execute","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from","get_causation_id","get_causation_id","get_causation_id","get_correlation_id","get_correlation_id","get_correlation_id","handle_recorded_event","identifier","into","into","into","into","into","serialize","started","stopped","stream_name","to_owned","to_owned","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","vzip","vzip","vzip","vzip","vzip"],"q":["chekov","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","chekov::aggregate","","","chekov::application","","","","","","","","","","","","","","","","","","chekov::event","","","","","","","","","","","","","","","","","","","","chekov::macros","","","","","","chekov::prelude","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"d":["Define an Aggregate","Application are high order logical seperator.","Struct to configure and launch an <code>Application</code> instance","Define a Command which can be dispatch","Define an Event which can be produced and consumed","The Event that can be generated for this command","Define an event applier","Define a struct as an EventHandler","The Executor that will execute the command and produce …","The registry where the command will be dispatched","The type of storage used by the application","Deal with Application subscriptions","Aggregates produce events based on commands","Struct and Trait correlated to Application","","","","","","","Struct and Trait correlated to Event","","","Returns the logical name of the application Mostly used …","","Returns the identifier for this command.","Define the identity of this kind of Aggregate.","","","","","","","","","Used to initiate the launch of the application","Define an Aggregate","","Define the identity of this kind of Aggregate.","Application are high order logical seperator.","Struct to configure and launch an <code>Application</code> instance","The type of storage used by the application","","","","Adds an EventHandler to the application","","Returns the logical name of the application Mostly used …","","Launch the application","Adds a preconfigured Backend as Storage. The storage isn…","","","","","Used to initiate the launch of the application","Adds a StorageConfig use later to create the event_store …","Define an Event which can be produced and consumed","Define an event applier","Define a struct as an EventHandler","Receive an immutable event to handle","","","","","","","","","","","","","","","","","","","","","","","Deals with the lifetime of a particular aggregate","","","Error returns by a failling EventApplier","Define a Command which can be dispatch","Receives a command and an immutable State and optionally …","Error returns by a CommandExecutor","","The Event that can be generated for this command","Define a struct as an EventHandler","","The Executor that will execute the command and produce …","The registry where the command will be dispatched","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Returns the optional causation id of the command","Returns the optional causation id of the command","Returns the optional causation id of the command","Returns the correlation id of the command","Returns the correlation id of the command","Returns the correlation id of the command","","Returns the identifier for this command.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"i":[0,0,0,0,0,1,0,0,1,1,2,0,0,0,0,3,4,5,5,5,0,0,5,2,6,1,4,5,7,0,5,5,5,5,5,2,0,4,4,0,0,2,8,8,8,8,8,2,8,8,8,8,8,8,8,2,8,0,0,0,0,3,6,6,6,6,9,6,7,6,6,6,6,6,6,6,6,0,0,0,0,0,0,0,10,11,0,0,0,0,0,1,0,0,1,1,12,13,10,11,14,12,13,10,11,14,13,14,13,14,13,14,13,13,13,14,12,13,10,15,12,13,10,11,14,12,13,10,10,11,14,1,1,1,1,1,1,6,1,12,13,10,11,14,10,12,12,14,13,14,12,13,10,11,14,12,13,10,11,14,12,13,10,11,14,12,13,10,11,14],"f":[null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[],[["result",4],["applyerror",4]]],[[["recordedevent",3]],[["result",4],["applyerror",4]]],[[]],[[]],[[]],null,null,[[]],[[],["str",15]],[[["recordedevent",3]],[["box",3],["pin",3]]],[[],["string",3]],[[],["str",15]],[[]],[[["recordedevent",3]],[["eventenvelope",3],["result",4]]],null,[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],[[],["applicationbuilder",3]],null,[[["recordedevent",3]],[["result",4],["applyerror",4]]],[[],["str",15]],null,null,null,[[]],[[]],[[]],[[["eventhandler",8]]],[[]],[[],["str",15]],[[]],[[]],[[["future",8]]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],[[],["applicationbuilder",3]],[[["storageconfig",8]]],null,null,null,null,[[],[["result",4],["applyerror",4]]],[[],["eventhandlerbuilder",3]],[[],["eventhandlerbuilder",3]],[[],["eventhandlerbuilder",3]],[[],["eventhandlerbuilder",3]],[[],[["boxfuture",6],["result",4]]],[[["recordedevent",3]],[["box",3],["pin",3]]],[[["recordedevent",3]],[["eventenvelope",3],["result",4]]],[[["context",3]]],[[["context",3]]],[[["context",3]]],[[["context",3]]],[[["context",3]]],[[["context",3]]],[[["context",3]]],[[["context",3]]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],null,null,[[],["commandmetadatas",3]],[[],["eventmetadatas",3]],[[]],[[]],null,null,null,null,[[],["aggregateinstance",3]],[[]],[[["serializer",8]],[["ok",3],["result",4],["error",3]]],[[],[["commandexecutorerror",4],["vec",3],["result",4]]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[["mailboxerror",4]]],[[]],[[]],[[]],[[],[["uuid",3],["option",4]]],[[],[["uuid",3],["option",4]]],[[],[["uuid",3],["option",4]]],[[],["uuid",3]],[[],["uuid",3]],[[],["uuid",3]],[[["recordedevent",3]],[["box",3],["pin",3]]],[[],["string",3]],[[]],[[]],[[]],[[]],[[]],[[],["result",4]],[[]],[[]],null,[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],[[]],[[]],[[]]],"p":[[8,"Command"],[8,"Application"],[8,"EventApplier"],[8,"Aggregate"],[3,"SubscriberManager"],[8,"EventHandler"],[8,"Event"],[3,"ApplicationBuilder"],[8,"Handler"],[4,"CommandExecutorError"],[4,"ApplyError"],[3,"AggregateInstance"],[3,"CommandMetadatas"],[3,"EventMetadatas"],[8,"CommandExecutor"]]},\
"chekov_macros":{"doc":"","t":[24,24,24,24,23,23],"n":["Aggregate","Command","Event","EventHandler","applier","event_handler"],"q":["chekov_macros","","","","",""],"d":["","","","","",""],"i":[0,0,0,0,0,0],"f":[null,null,null,null,null,null],"p":[]},\
"event_store":{"doc":"The <code>event_store</code> crate","t":[8,3,3,10,5,11,11,11,11,11,11,11,11,11,10,11,11,11,11,11,11,0,5,11,11,11,11,11,11,11,11,11,11,13,13,3,8,4,4,3,13,13,3,4,3,3,3,8,13,4,3,13,13,13,3,13,3,13,13,10,10,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,11,11,11,11,11,11,11,11,12,12,10,11,11,12,11,11,11,11,10,11,11,11,11,11,11,11,11,10,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,10,11,11,10,11,11,11,10,11,11,11,11,12,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11],"n":["Event","EventStore","EventStoreBuilder","all_event_types","append","borrow","borrow","borrow_mut","borrow_mut","build","builder","clone","clone_into","default","event_type","fmt","from","from","handle","into","into","prelude","read","storage","to_owned","try_from","try_from","try_into","try_into","type_id","type_id","vzip","vzip","Any","AnyVersion","Appender","Event","EventStoreError","ExpectedVersion","InMemoryBackend","NoStream","Origin","PostgresBackend","ReadVersion","Reader","RecordedEvent","RecordedEvents","Storage","Storage","StorageError","Stream","StreamAlreadyExists","StreamDoesntExists","StreamExists","StreamInfo","Unknown","UnsavedEvent","Version","Version","all_event_types","append_to_stream","append_to_stream","append_to_stream","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","causation_id","clone","clone","clone","clone","clone_into","clone_into","clone_into","clone_into","correlation_id","correlation_id","create_stream","create_stream","create_stream","data","default","default","default","default","delete_stream","delete_stream","delete_stream","eq","eq","eq","eq","eq","event","event_type","event_type","event_type_from_str","event_type_from_str","event_type_from_str","events","execute","execute_async","expected_version","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from_row","from_str","into","into","into","into","into","into","into","into","into","into","into","into","into","limit","ne","ne","ne","ne","read_stream","read_stream","read_stream","read_stream_info","read_stream_info","read_stream_info","serialize","storage_name","storage_name","storage_name","stream","stream_uuid","stream_uuid","stream_uuid","to","to_owned","to_owned","to_owned","to_owned","to_string","try_deserialize","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","verify","vzip","vzip","vzip","vzip","vzip","vzip","vzip","vzip","vzip","vzip","vzip","vzip","vzip","with_correlation_id","with_correlation_id","with_url"],"q":["event_store","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","event_store::prelude","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"d":["Represent event that can be handled by an <code>EventStore</code>","An <code>EventStore</code> that hold a storage connection","Builder use to simplify the <code>EventStore</code> creation","","Create an <code>Appender</code> to append events","","","","","Try to build the previously configured <code>EventStore</code>","","","","","Return a static str which define the event type","","","","","","","","Create a <code>Reader</code> to read a stream","Define which storage will be used by this building …","","","","","","","","","","","Define that we expect a stream in any version","An Appender defines the parameters of an append action","Represent event that can be handled by an <code>EventStore</code>","","The <code>ExpectedVersion</code> used to define optimistic concurrency","","Define that we expect a non existing stream","","","","","A <code>RecordedEvent</code> represents an <code>Event</code> which have been …","","A <code>Storage</code> is responsible for storing and managing <code>Stream</code> …","","","A <code>Stream</code> represents an <code>Event</code> stream","","","Define that we expect an existing stream","","","An <code>UnsavedEvent</code> is created from a type that implement …","Define that we expect a stream in a particular version","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","a <code>causation_id</code> defines who caused this event","","","","","","","","","","a <code>correlation_id</code> correlates multiple events","Create a new stream with an identifier","","","Payload of this event","","","","","Delete a stream from the <code>Backend</code>","","","","","","","","Add a single <code>Event</code>s to the <code>Appender</code>","Return a static str which define the event type","Human readable event type","","","","Add a list of <code>Event</code>s to the <code>Appender</code>","Execute the <code>Appender</code> against a <code>Storage</code> and returns the …","","Define the expected version of the stream we are …","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Define which stream we are reading from","","","The stream identifier for thie event","Define which stream we are appending to","","","","","","Errors","","","","","","","","Errors","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Errors"],"i":[0,0,0,1,0,2,3,2,3,3,2,2,2,2,1,3,2,3,2,2,3,0,0,3,2,2,3,2,3,2,3,2,3,4,5,0,0,0,0,0,5,6,0,0,0,0,0,0,4,0,0,7,7,5,0,7,0,5,6,1,8,9,10,11,9,12,13,4,14,15,16,5,6,10,7,17,11,9,12,13,4,14,15,16,5,6,10,7,17,14,14,15,16,17,14,15,16,17,13,14,8,9,10,14,11,9,12,10,8,9,10,4,16,5,7,17,11,1,14,1,1,1,11,11,12,11,9,4,4,14,15,16,5,6,10,7,17,11,9,12,12,13,4,4,14,15,16,5,6,10,7,7,17,14,17,11,9,12,13,4,14,15,16,5,6,10,7,17,12,4,16,5,17,8,9,10,8,9,10,14,8,9,10,12,17,13,14,11,14,15,16,17,4,14,11,9,12,13,4,14,15,16,16,5,6,10,7,17,11,9,12,13,4,14,15,16,5,6,10,7,17,11,9,12,13,4,14,15,16,5,6,10,7,17,5,11,9,12,13,4,14,15,16,5,6,10,7,17,11,12,9],"f":[null,null,null,[[],[["str",15],["vec",3]]],[[],["appender",3]],[[]],[[]],[[]],[[]],[[]],[[],["eventstorebuilder",3]],[[],["eventstore",3]],[[]],[[]],[[],["str",15]],[[["formatter",3]],["result",6]],[[]],[[]],[[["streaminfo",3]]],[[]],[[]],null,[[],["reader",3]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[],[["str",15],["vec",3]]],[[["uuid",3],["str",15]],[["box",3],["pin",3]]],[[["uuid",3],["str",15]],[["box",3],["pin",3]]],[[["uuid",3],["str",15]],[["box",3],["pin",3]]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],null,[[],["recordedevent",3]],[[],["recordedevents",3]],[[],["unsavedevent",3]],[[],["stream",3]],[[]],[[]],[[]],[[]],null,null,[[["uuid",3],["stream",3]],[["pin",3],["box",3]]],[[["uuid",3],["stream",3]],[["pin",3],["box",3]]],[[["uuid",3],["stream",3]],[["pin",3],["box",3]]],null,[[]],[[]],[[]],[[],["inmemorybackend",3]],[[["stream",3],["uuid",3]],[["pin",3],["box",3]]],[[["stream",3],["uuid",3]],[["pin",3],["box",3]]],[[["stream",3],["uuid",3]],[["pin",3],["box",3]]],[[["eventstoreerror",4]],["bool",15]],[[["unsavedevent",3]],["bool",15]],[[["expectedversion",4]],["bool",15]],[[["storageerror",4]],["bool",15]],[[["stream",3]],["bool",15]],[[],[["eventstoreerror",4],["result",4]]],[[],["str",15]],null,[[],["str",15]],[[],["str",15]],[[],["str",15]],[[],[["eventstoreerror",4],["result",4]]],[[["eventstore",3],["addr",3]]],[[["eventstore",3],["addr",3]]],[[["expectedversion",4]]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[["readversion",4]]],[[]],[[]],[[["mailboxerror",4]]],[[]],[[]],[[]],[[]],[[]],[[]],[[["error",4]]],[[]],[[]],[[],["result",6]],[[["str",15]],["result",4]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[["usize",15]]],[[["eventstoreerror",4]],["bool",15]],[[["unsavedevent",3]],["bool",15]],[[["expectedversion",4]],["bool",15]],[[["stream",3]],["bool",15]],[[["usize",15],["uuid",3],["string",3]],[["box",3],["pin",3]]],[[["usize",15],["uuid",3],["string",3]],[["box",3],["pin",3]]],[[["usize",15],["uuid",3],["string",3]],[["box",3],["pin",3]]],[[["uuid",3],["string",3]],[["pin",3],["box",3]]],[[["uuid",3],["string",3]],[["pin",3],["box",3]]],[[["uuid",3],["string",3]],[["pin",3],["box",3]]],[[],["result",4]],[[],["str",15]],[[],["str",15]],[[],["str",15]],[[["into",8],["string",3]],[["eventstoreerror",4],["result",4]]],[[],["str",15]],null,null,[[],[["eventstoreerror",4],["result",4]]],[[]],[[]],[[]],[[]],[[],["string",3]],[[],[["result",4],["recordedeventerror",4],["deserialize",8],["event",8],["deserialize",8]]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],[["result",4],["parseeventerror",4]]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[["stream",3]],["expectedversionresult",4]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[["uuid",3]]],[[["uuid",3]]],[[["str",15]]]],"p":[[8,"Event"],[3,"EventStore"],[3,"EventStoreBuilder"],[4,"EventStoreError"],[4,"ExpectedVersion"],[4,"ReadVersion"],[4,"StorageError"],[8,"Storage"],[3,"PostgresBackend"],[3,"InMemoryBackend"],[3,"Appender"],[3,"Reader"],[3,"StreamInfo"],[3,"RecordedEvent"],[3,"RecordedEvents"],[3,"UnsavedEvent"],[3,"Stream"]]}\
}');
if (window.initSearch) {window.initSearch(searchIndex)};