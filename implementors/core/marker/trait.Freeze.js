(function() {var implementors = {};
implementors["chekov"] = [{"text":"impl&lt;A&gt; Freeze for ApplicationBuilder&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; Freeze for SubscriberManager&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; Freeze for AggregateInstance&lt;A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; Freeze for DefaultEventResolver&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for CommandMetadatas","synthetic":true,"types":[]},{"text":"impl Freeze for CommandExecutorError","synthetic":true,"types":[]},{"text":"impl Freeze for ApplyError","synthetic":true,"types":[]},{"text":"impl&lt;A, E&gt; Freeze for EventHandlerInstance&lt;A, E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for EventMetadatas","synthetic":true,"types":[]}];
implementors["event_store"] = [{"text":"impl&lt;S&gt; Freeze for EventStore&lt;S&gt;","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Freeze for EventStoreBuilder&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for StreamInfo","synthetic":true,"types":[]},{"text":"impl Freeze for EventStoreError","synthetic":true,"types":[]},{"text":"impl Freeze for RecordedEvent","synthetic":true,"types":[]},{"text":"impl Freeze for RecordedEvents","synthetic":true,"types":[]},{"text":"impl Freeze for UnsavedEvent","synthetic":true,"types":[]},{"text":"impl Freeze for ExpectedVersion","synthetic":true,"types":[]},{"text":"impl Freeze for ReadVersion","synthetic":true,"types":[]},{"text":"impl Freeze for StorageError","synthetic":true,"types":[]},{"text":"impl Freeze for Appender","synthetic":true,"types":[]},{"text":"impl Freeze for InMemoryBackend","synthetic":true,"types":[]},{"text":"impl Freeze for PostgresBackend","synthetic":true,"types":[]},{"text":"impl Freeze for Reader","synthetic":true,"types":[]},{"text":"impl Freeze for Stream","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()