(function() {var implementors = {};
implementors["chekov"] = [{"text":"impl&lt;A&gt; Unpin for ApplicationBuilder&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; Unpin for SubscriberManager&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; Unpin for AggregateInstance&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; Unpin for DefaultEventResolver&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for CommandMetadatas","synthetic":true,"types":[]},{"text":"impl Unpin for CommandExecutorError","synthetic":true,"types":[]},{"text":"impl Unpin for ApplyError","synthetic":true,"types":[]},{"text":"impl&lt;A, E&gt; Unpin for EventHandlerInstance&lt;A, E&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for EventMetadatas","synthetic":true,"types":[]}];
implementors["event_store"] = [{"text":"impl&lt;S&gt; Unpin for EventStore&lt;S&gt;","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for EventStoreBuilder&lt;S&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for StreamInfo","synthetic":true,"types":[]},{"text":"impl Unpin for EventStoreError","synthetic":true,"types":[]},{"text":"impl Unpin for RecordedEvent","synthetic":true,"types":[]},{"text":"impl Unpin for RecordedEvents","synthetic":true,"types":[]},{"text":"impl Unpin for UnsavedEvent","synthetic":true,"types":[]},{"text":"impl Unpin for ExpectedVersion","synthetic":true,"types":[]},{"text":"impl Unpin for ReadVersion","synthetic":true,"types":[]},{"text":"impl Unpin for StorageError","synthetic":true,"types":[]},{"text":"impl Unpin for Appender","synthetic":true,"types":[]},{"text":"impl Unpin for InMemoryBackend","synthetic":true,"types":[]},{"text":"impl Unpin for PostgresBackend","synthetic":true,"types":[]},{"text":"impl Unpin for Reader","synthetic":true,"types":[]},{"text":"impl Unpin for Stream","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()