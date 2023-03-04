var sourcesIndex = JSON.parse('{\
"chekov":["",[["aggregate",[["instance",[],["internal.rs","mod.rs","runtime.rs"]]],["mod.rs","registry.rs","resolver.rs"]],["application",[],["builder.rs","internal.rs","mod.rs"]],["command",[["handler",[],["instance.rs","mod.rs","registry.rs"]]],["consistency.rs","metadata.rs","mod.rs"]],["event",[],["handler.rs","mod.rs","resolver.rs"]],["subscriber",[],["listener.rs","manager.rs","mod.rs","subscriber.rs"]]],["error.rs","event_store.rs","lib.rs","message.rs","prelude.rs","router.rs"]],\
"chekov_api":["",[["generated",[],["rs.chekov.api.aggregates.rs","rs.chekov.api.chekov.rs","rs.chekov.api.events.rs","rs.chekov.api.streams.rs"]]],["aggregates.rs","client.rs","events.rs","lib.rs","streams.rs"]],\
"chekov_macros":["",[],["aggregate.rs","command.rs","event.rs","event_handler.rs","lib.rs"]],\
"console_subscriber":["",[],["builder.rs","command.rs","lib.rs","server.rs","visitors.rs"]],\
"event_store":["",[["connection",[],["messaging.rs","mod.rs"]],["event",[],["mod.rs"]],["event_store",[],["logic.rs","mod.rs","runtime.rs"]],["storage",[],["appender.rs","mod.rs","reader.rs"]],["subscriptions",[],["error.rs","fsm.rs","mod.rs","pub_sub.rs","state.rs","subscriber.rs","subscription.rs","supervisor.rs"]]],["lib.rs","prelude.rs"]],\
"event_store_backend_inmemory":["",[],["lib.rs"]],\
"event_store_backend_postgres":["",[],["error.rs","lib.rs","sql.rs"]],\
"event_store_core":["",[["backend",[],["error.rs","mod.rs"]],["event",[],["error.rs","mod.rs"]],["event_bus",[],["error.rs","mod.rs"]],["storage",[],["error.rs","mod.rs"]],["stream",[],["error.rs","mod.rs"]]],["error.rs","lib.rs","versions.rs"]],\
"event_store_eventbus_inmemory":["",[],["lib.rs"]],\
"event_store_eventbus_postgres":["",[],["lib.rs"]],\
"event_store_storage_inmemory":["",[],["lib.rs"]],\
"event_store_storage_postgres":["",[],["lib.rs"]],\
"gift_shop":["",[["account",[],["aggregate.rs","projector.rs","repository.rs"]]],["account.rs","commands.rs","events.rs","gift_card.rs","http.rs","main.rs","order.rs"]],\
"watcher":["",[["ui",[],["util.rs"]]],["app.rs","conn.rs","main.rs","state.rs","ui.rs"]]\
}');
createSourceSidebar();
