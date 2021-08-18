(function() {var implementors = {};
implementors["chekov"] = [{"text":"impl&lt;A&gt; Freeze for <a class=\"struct\" href=\"chekov/aggregate/struct.AggregateInstance.html\" title=\"struct chekov::aggregate::AggregateInstance\">AggregateInstance</a>&lt;A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Freeze,&nbsp;</span>","synthetic":true,"types":["chekov::aggregate::instance::AggregateInstance"]},{"text":"impl&lt;A&gt; Freeze for <a class=\"struct\" href=\"chekov/application/struct.ApplicationBuilder.html\" title=\"struct chekov::application::ApplicationBuilder\">ApplicationBuilder</a>&lt;A&gt;","synthetic":true,"types":["chekov::application::builder::ApplicationBuilder"]},{"text":"impl&lt;A&gt; Freeze for <a class=\"struct\" href=\"chekov/application/struct.DefaultEventResolver.html\" title=\"struct chekov::application::DefaultEventResolver\">DefaultEventResolver</a>&lt;A&gt;","synthetic":true,"types":["chekov::application::DefaultEventResolver"]},{"text":"impl Freeze for <a class=\"struct\" href=\"chekov/prelude/struct.CommandMetadatas.html\" title=\"struct chekov::prelude::CommandMetadatas\">CommandMetadatas</a>","synthetic":true,"types":["chekov::command::metadata::CommandMetadatas"]},{"text":"impl Freeze for <a class=\"enum\" href=\"chekov/prelude/enum.CommandExecutorError.html\" title=\"enum chekov::prelude::CommandExecutorError\">CommandExecutorError</a>","synthetic":true,"types":["chekov::error::CommandExecutorError"]},{"text":"impl Freeze for <a class=\"enum\" href=\"chekov/prelude/enum.ApplyError.html\" title=\"enum chekov::prelude::ApplyError\">ApplyError</a>","synthetic":true,"types":["chekov::error::ApplyError"]},{"text":"impl&lt;A, E&gt; Freeze for <a class=\"struct\" href=\"chekov/event/struct.EventHandlerInstance.html\" title=\"struct chekov::event::EventHandlerInstance\">EventHandlerInstance</a>&lt;A, E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: Freeze,&nbsp;</span>","synthetic":true,"types":["chekov::event::handler::EventHandlerInstance"]},{"text":"impl Freeze for <a class=\"struct\" href=\"chekov/prelude/struct.EventMetadatas.html\" title=\"struct chekov::prelude::EventMetadatas\">EventMetadatas</a>","synthetic":true,"types":["chekov::message::EventMetadatas"]},{"text":"impl&lt;A&gt; Freeze for <a class=\"struct\" href=\"chekov/struct.SubscriberManager.html\" title=\"struct chekov::SubscriberManager\">SubscriberManager</a>&lt;A&gt;","synthetic":true,"types":["chekov::subscriber::manager::SubscriberManager"]}];
implementors["event_store"] = [{"text":"impl Freeze for <a class=\"struct\" href=\"event_store/prelude/struct.StreamInfo.html\" title=\"struct event_store::prelude::StreamInfo\">StreamInfo</a>","synthetic":true,"types":["event_store::connection::messaging::StreamInfo"]},{"text":"impl Freeze for <a class=\"enum\" href=\"event_store/prelude/enum.EventStoreError.html\" title=\"enum event_store::prelude::EventStoreError\">EventStoreError</a>","synthetic":true,"types":["event_store::error::EventStoreError"]},{"text":"impl Freeze for <a class=\"struct\" href=\"event_store/prelude/struct.RecordedEvent.html\" title=\"struct event_store::prelude::RecordedEvent\">RecordedEvent</a>","synthetic":true,"types":["event_store::event::recorded::RecordedEvent"]},{"text":"impl Freeze for <a class=\"struct\" href=\"event_store/prelude/struct.RecordedEvents.html\" title=\"struct event_store::prelude::RecordedEvents\">RecordedEvents</a>","synthetic":true,"types":["event_store::event::recorded::RecordedEvents"]},{"text":"impl Freeze for <a class=\"struct\" href=\"event_store/prelude/struct.UnsavedEvent.html\" title=\"struct event_store::prelude::UnsavedEvent\">UnsavedEvent</a>","synthetic":true,"types":["event_store::event::unsaved::UnsavedEvent"]},{"text":"impl Freeze for <a class=\"enum\" href=\"event_store/prelude/enum.ExpectedVersion.html\" title=\"enum event_store::prelude::ExpectedVersion\">ExpectedVersion</a>","synthetic":true,"types":["event_store::expected_version::ExpectedVersion"]},{"text":"impl Freeze for <a class=\"enum\" href=\"event_store/prelude/enum.ReadVersion.html\" title=\"enum event_store::prelude::ReadVersion\">ReadVersion</a>","synthetic":true,"types":["event_store::read_version::ReadVersion"]},{"text":"impl Freeze for <a class=\"struct\" href=\"event_store/prelude/struct.Appender.html\" title=\"struct event_store::prelude::Appender\">Appender</a>","synthetic":true,"types":["event_store::storage::appender::Appender"]},{"text":"impl Freeze for <a class=\"struct\" href=\"event_store/prelude/struct.InMemoryBackend.html\" title=\"struct event_store::prelude::InMemoryBackend\">InMemoryBackend</a>","synthetic":true,"types":["event_store::storage::inmemory::InMemoryBackend"]},{"text":"impl Freeze for <a class=\"struct\" href=\"event_store/prelude/struct.PostgresBackend.html\" title=\"struct event_store::prelude::PostgresBackend\">PostgresBackend</a>","synthetic":true,"types":["event_store::storage::postgres::PostgresBackend"]},{"text":"impl Freeze for <a class=\"struct\" href=\"event_store/prelude/struct.Reader.html\" title=\"struct event_store::prelude::Reader\">Reader</a>","synthetic":true,"types":["event_store::storage::reader::Reader"]},{"text":"impl Freeze for <a class=\"enum\" href=\"event_store/prelude/enum.StorageError.html\" title=\"enum event_store::prelude::StorageError\">StorageError</a>","synthetic":true,"types":["event_store::storage::StorageError"]},{"text":"impl Freeze for <a class=\"struct\" href=\"event_store/prelude/struct.Stream.html\" title=\"struct event_store::prelude::Stream\">Stream</a>","synthetic":true,"types":["event_store::stream::Stream"]},{"text":"impl&lt;S&gt; Freeze for <a class=\"struct\" href=\"event_store/struct.EventStore.html\" title=\"struct event_store::EventStore\">EventStore</a>&lt;S&gt;","synthetic":true,"types":["event_store::EventStore"]},{"text":"impl&lt;S&gt; Freeze for <a class=\"struct\" href=\"event_store/struct.EventStoreBuilder.html\" title=\"struct event_store::EventStoreBuilder\">EventStoreBuilder</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":["event_store::EventStoreBuilder"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()