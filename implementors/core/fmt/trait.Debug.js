(function() {var implementors = {};
implementors["chekov"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"chekov/command/enum.Consistency.html\" title=\"enum chekov::command::Consistency\">Consistency</a>","synthetic":false,"types":["chekov::command::consistency::Consistency"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"chekov/command/struct.CommandMetadatas.html\" title=\"struct chekov::command::CommandMetadatas\">CommandMetadatas</a>","synthetic":false,"types":["chekov::command::metadata::CommandMetadatas"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"chekov/prelude/enum.CommandExecutorError.html\" title=\"enum chekov::prelude::CommandExecutorError\">CommandExecutorError</a>","synthetic":false,"types":["chekov::error::CommandExecutorError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"chekov/prelude/enum.ApplyError.html\" title=\"enum chekov::prelude::ApplyError\">ApplyError</a>","synthetic":false,"types":["chekov::error::ApplyError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"chekov/prelude/struct.EventMetadatas.html\" title=\"struct chekov::prelude::EventMetadatas\">EventMetadatas</a>","synthetic":false,"types":["chekov::message::EventMetadatas"]}];
implementors["event_store"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"event_store/prelude/enum.EventStoreError.html\" title=\"enum event_store::prelude::EventStoreError\">EventStoreError</a>","synthetic":false,"types":["event_store::error::EventStoreError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store/prelude/struct.RecordedEvents.html\" title=\"struct event_store::prelude::RecordedEvents\">RecordedEvents</a>","synthetic":false,"types":["event_store::event::RecordedEvents"]},{"text":"impl&lt;S:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"event_store_core/storage/trait.Storage.html\" title=\"trait event_store_core::storage::Storage\">Storage</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store/prelude/struct.EventStore.html\" title=\"struct event_store::prelude::EventStore\">EventStore</a>&lt;S&gt;","synthetic":false,"types":["event_store::event_store::EventStore"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store/storage/appender/struct.AppendToStreamRequest.html\" title=\"struct event_store::storage::appender::AppendToStreamRequest\">AppendToStreamRequest</a>","synthetic":false,"types":["event_store::storage::appender::AppendToStreamRequest"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store/storage/reader/struct.ReadStreamRequest.html\" title=\"struct event_store::storage::reader::ReadStreamRequest\">ReadStreamRequest</a>","synthetic":false,"types":["event_store::storage::reader::ReadStreamRequest"]},{"text":"impl&lt;S:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"event_store_core/storage/trait.Storage.html\" title=\"trait event_store_core::storage::Storage\">Storage</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store/prelude/struct.Subscription.html\" title=\"struct event_store::prelude::Subscription\">Subscription</a>&lt;S&gt;","synthetic":false,"types":["event_store::subscriptions::subscription::Subscription"]},{"text":"impl&lt;S:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"event_store_core/storage/trait.Storage.html\" title=\"trait event_store_core::storage::Storage\">Storage</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store/prelude/struct.SubscriptionsSupervisor.html\" title=\"struct event_store::prelude::SubscriptionsSupervisor\">SubscriptionsSupervisor</a>&lt;S&gt;","synthetic":false,"types":["event_store::subscriptions::supervisor::SubscriptionsSupervisor"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store/prelude/struct.SubscriptionOptions.html\" title=\"struct event_store::prelude::SubscriptionOptions\">SubscriptionOptions</a>","synthetic":false,"types":["event_store::subscriptions::SubscriptionOptions"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"event_store/prelude/enum.StartFrom.html\" title=\"enum event_store::prelude::StartFrom\">StartFrom</a>","synthetic":false,"types":["event_store::subscriptions::StartFrom"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"event_store/prelude/enum.SubscriptionNotification.html\" title=\"enum event_store::prelude::SubscriptionNotification\">SubscriptionNotification</a>","synthetic":false,"types":["event_store::subscriptions::SubscriptionNotification"]}];
implementors["event_store_backend_inmemory"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store_backend_inmemory/struct.InMemoryBackend.html\" title=\"struct event_store_backend_inmemory::InMemoryBackend\">InMemoryBackend</a>","synthetic":false,"types":["event_store_backend_inmemory::InMemoryBackend"]}];
implementors["event_store_backend_postgres"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store_backend_postgres/struct.PostgresBackend.html\" title=\"struct event_store_backend_postgres::PostgresBackend\">PostgresBackend</a>","synthetic":false,"types":["event_store_backend_postgres::PostgresBackend"]}];
implementors["event_store_core"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store_core/event/struct.RecordedEvent.html\" title=\"struct event_store_core::event::RecordedEvent\">RecordedEvent</a>","synthetic":false,"types":["event_store_core::event::RecordedEvent"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store_core/event/struct.UnsavedEvent.html\" title=\"struct event_store_core::event::UnsavedEvent\">UnsavedEvent</a>","synthetic":false,"types":["event_store_core::event::UnsavedEvent"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"event_store_core/event/enum.ParseEventError.html\" title=\"enum event_store_core::event::ParseEventError\">ParseEventError</a>","synthetic":false,"types":["event_store_core::event::ParseEventError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"event_store_core/event_bus/enum.EventBusMessage.html\" title=\"enum event_store_core::event_bus::EventBusMessage\">EventBusMessage</a>","synthetic":false,"types":["event_store_core::event_bus::EventBusMessage"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store_core/event_bus/struct.EventNotification.html\" title=\"struct event_store_core::event_bus::EventNotification\">EventNotification</a>","synthetic":false,"types":["event_store_core::event_bus::EventNotification"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"event_store_core/storage/enum.StorageError.html\" title=\"enum event_store_core::storage::StorageError\">StorageError</a>","synthetic":false,"types":["event_store_core::storage::StorageError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store_core/stream/struct.Stream.html\" title=\"struct event_store_core::stream::Stream\">Stream</a>","synthetic":false,"types":["event_store_core::stream::Stream"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"event_store_core/stream/enum.StreamError.html\" title=\"enum event_store_core::stream::StreamError\">StreamError</a>","synthetic":false,"types":["event_store_core::stream::StreamError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"event_store_core/versions/enum.ReadVersion.html\" title=\"enum event_store_core::versions::ReadVersion\">ReadVersion</a>","synthetic":false,"types":["event_store_core::versions::ReadVersion"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"event_store_core/versions/enum.ExpectedVersion.html\" title=\"enum event_store_core::versions::ExpectedVersion\">ExpectedVersion</a>","synthetic":false,"types":["event_store_core::versions::ExpectedVersion"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"event_store_core/versions/enum.ExpectedVersionResult.html\" title=\"enum event_store_core::versions::ExpectedVersionResult\">ExpectedVersionResult</a>","synthetic":false,"types":["event_store_core::versions::ExpectedVersionResult"]}];
implementors["event_store_eventbus_inmemory"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store_eventbus_inmemory/struct.InMemoryEventBus.html\" title=\"struct event_store_eventbus_inmemory::InMemoryEventBus\">InMemoryEventBus</a>","synthetic":false,"types":["event_store_eventbus_inmemory::InMemoryEventBus"]}];
implementors["event_store_eventbus_postgres"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store_eventbus_postgres/struct.PostgresEventBus.html\" title=\"struct event_store_eventbus_postgres::PostgresEventBus\">PostgresEventBus</a>","synthetic":false,"types":["event_store_eventbus_postgres::PostgresEventBus"]}];
implementors["event_store_storage_inmemory"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store_storage_inmemory/struct.InMemoryStorage.html\" title=\"struct event_store_storage_inmemory::InMemoryStorage\">InMemoryStorage</a>","synthetic":false,"types":["event_store_storage_inmemory::InMemoryStorage"]}];
implementors["event_store_storage_postgres"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"event_store_storage_postgres/struct.PostgresStorage.html\" title=\"struct event_store_storage_postgres::PostgresStorage\">PostgresStorage</a>","synthetic":false,"types":["event_store_storage_postgres::PostgresStorage"]}];
implementors["gift_shop"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"gift_shop/account/projector/struct.ACCOUNTPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER.html\" title=\"struct gift_shop::account::projector::ACCOUNTPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER\">ACCOUNTPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER</a>","synthetic":false,"types":["gift_shop::account::projector::ACCOUNTPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"gift_shop/account/enum.AccountStatus.html\" title=\"enum gift_shop::account::AccountStatus\">AccountStatus</a>","synthetic":false,"types":["gift_shop::account::AccountStatus"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"gift_shop/account/struct.Account.html\" title=\"struct gift_shop::account::Account\">Account</a>","synthetic":false,"types":["gift_shop::account::Account"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"gift_shop/account/struct.ACCOUNT_STATIC_EVENT_RESOLVER.html\" title=\"struct gift_shop::account::ACCOUNT_STATIC_EVENT_RESOLVER\">ACCOUNT_STATIC_EVENT_RESOLVER</a>","synthetic":false,"types":["gift_shop::account::ACCOUNT_STATIC_EVENT_RESOLVER"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"gift_shop/gift_card/enum.GiftCardState.html\" title=\"enum gift_shop::gift_card::GiftCardState\">GiftCardState</a>","synthetic":false,"types":["gift_shop::gift_card::GiftCardState"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"gift_shop/gift_card/struct.GiftCard.html\" title=\"struct gift_shop::gift_card::GiftCard\">GiftCard</a>","synthetic":false,"types":["gift_shop::gift_card::GiftCard"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"gift_shop/gift_card/struct.GIFTCARD_STATIC_EVENT_RESOLVER.html\" title=\"struct gift_shop::gift_card::GIFTCARD_STATIC_EVENT_RESOLVER\">GIFTCARD_STATIC_EVENT_RESOLVER</a>","synthetic":false,"types":["gift_shop::gift_card::GIFTCARD_STATIC_EVENT_RESOLVER"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"gift_shop/gift_card/struct.GIFTCARDPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER.html\" title=\"struct gift_shop::gift_card::GIFTCARDPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER\">GIFTCARDPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER</a>","synthetic":false,"types":["gift_shop::gift_card::GIFTCARDPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"gift_shop/order/enum.OrderStatus.html\" title=\"enum gift_shop::order::OrderStatus\">OrderStatus</a>","synthetic":false,"types":["gift_shop::order::OrderStatus"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"gift_shop/order/struct.Item.html\" title=\"struct gift_shop::order::Item\">Item</a>","synthetic":false,"types":["gift_shop::order::Item"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"gift_shop/order/struct.Order.html\" title=\"struct gift_shop::order::Order\">Order</a>","synthetic":false,"types":["gift_shop::order::Order"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"gift_shop/order/struct.ORDER_STATIC_EVENT_RESOLVER.html\" title=\"struct gift_shop::order::ORDER_STATIC_EVENT_RESOLVER\">ORDER_STATIC_EVENT_RESOLVER</a>","synthetic":false,"types":["gift_shop::order::ORDER_STATIC_EVENT_RESOLVER"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"gift_shop/order/struct.ORDERPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER.html\" title=\"struct gift_shop::order::ORDERPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER\">ORDERPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER</a>","synthetic":false,"types":["gift_shop::order::ORDERPROJECTOR_STATIC_EVENT_HANDLER_EVENT_RESOLVER"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()