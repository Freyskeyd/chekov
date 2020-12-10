(function() {var implementors = {};
implementors["bank"] = [{"text":"impl RefUnwindSafe for DefaultApp","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for Account","synthetic":true,"types":[]},{"text":"impl !RefUnwindSafe for AccountEventRegistration","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for AccountStatus","synthetic":true,"types":[]},{"text":"impl !RefUnwindSafe for AccountProjector","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for DeleteAccount","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for OpenAccount","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for UpdateAccount","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for AccountDeleted","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for AccountOpened","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for UserRegistered","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for AccountUpdated","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for MoneyMovementEvent","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for find_all","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for find","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for create","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for update","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for delete","synthetic":true,"types":[]}];
implementors["chekov"] = [{"text":"impl&lt;A&gt; !RefUnwindSafe for SubscriberManager&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; RefUnwindSafe for AggregateInstance&lt;A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; !RefUnwindSafe for AggregateInstanceRegistry&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; !RefUnwindSafe for ApplicationBuilder&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; !RefUnwindSafe for DefaultEventResolver&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for CommandMetadatas","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for CommandExecutorError","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for ApplyError","synthetic":true,"types":[]},{"text":"impl&lt;A, E&gt; RefUnwindSafe for EventHandlerInstance&lt;A, E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: RefUnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;E: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; !RefUnwindSafe for Subscribe&lt;M&gt;","synthetic":true,"types":[]},{"text":"impl&lt;C, A&gt; RefUnwindSafe for Dispatch&lt;C, A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: RefUnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for EventMetadatas","synthetic":true,"types":[]},{"text":"impl&lt;E&gt; RefUnwindSafe for EventEnvelope&lt;E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; RefUnwindSafe for Router&lt;A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]}];
implementors["event_store"] = [{"text":"impl&lt;S&gt; !RefUnwindSafe for EventStore&lt;S&gt;","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; RefUnwindSafe for EventStoreBuilder&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for StreamInfo","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for EventStoreError","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for RecordedEvent","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for RecordedEvents","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for UnsavedEvent","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for ExpectedVersion","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for ReadVersion","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for StorageError","synthetic":true,"types":[]},{"text":"impl !RefUnwindSafe for Appender","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for InMemoryBackend","synthetic":true,"types":[]},{"text":"impl !RefUnwindSafe for PostgresBackend","synthetic":true,"types":[]},{"text":"impl !RefUnwindSafe for Reader","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for Stream","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()