var N = null;var sourcesIndex = {};
sourcesIndex["chekov"] = {"name":"","dirs":[{"name":"aggregate","dirs":[{"name":"instance","files":["internal.rs","mod.rs","runtime.rs"]}],"files":["mod.rs","registry.rs","resolver.rs"]},{"name":"application","files":["builder.rs","internal.rs","mod.rs"]},{"name":"command","dirs":[{"name":"handler","files":["instance.rs","mod.rs","registry.rs"]}],"files":["consistency.rs","metadata.rs","mod.rs"]},{"name":"event","files":["handler.rs","mod.rs","resolver.rs"]},{"name":"subscriber","files":["listener.rs","manager.rs","mod.rs","subscriber.rs"]}],"files":["error.rs","event_store.rs","lib.rs","message.rs","prelude.rs","router.rs"]};
sourcesIndex["chekov_macros"] = {"name":"","files":["aggregate.rs","command.rs","event.rs","event_handler.rs","lib.rs"]};
sourcesIndex["event_store"] = {"name":"","dirs":[{"name":"connection","files":["messaging.rs","mod.rs"]},{"name":"event","files":["mod.rs"]},{"name":"event_store","files":["logic.rs","mod.rs","runtime.rs"]},{"name":"storage","files":["appender.rs","mod.rs","reader.rs"]},{"name":"subscriptions","files":["error.rs","fsm.rs","mod.rs","pub_sub.rs","state.rs","subscriber.rs","subscription.rs","supervisor.rs"]}],"files":["lib.rs","prelude.rs"]};
sourcesIndex["event_store_backend_inmemory"] = {"name":"","files":["lib.rs"]};
sourcesIndex["event_store_backend_postgres"] = {"name":"","files":["error.rs","lib.rs","sql.rs"]};
sourcesIndex["event_store_core"] = {"name":"","dirs":[{"name":"backend","files":["error.rs","mod.rs"]},{"name":"event","files":["error.rs","mod.rs"]},{"name":"event_bus","files":["error.rs","mod.rs"]},{"name":"storage","files":["error.rs","mod.rs"]},{"name":"stream","files":["error.rs","mod.rs"]}],"files":["error.rs","lib.rs","versions.rs"]};
sourcesIndex["event_store_eventbus_inmemory"] = {"name":"","files":["lib.rs"]};
sourcesIndex["event_store_eventbus_postgres"] = {"name":"","files":["lib.rs"]};
sourcesIndex["event_store_storage_inmemory"] = {"name":"","files":["lib.rs"]};
sourcesIndex["event_store_storage_postgres"] = {"name":"","files":["lib.rs"]};
sourcesIndex["gift_shop"] = {"name":"","dirs":[{"name":"account","files":["aggregate.rs","projector.rs","repository.rs"]}],"files":["account.rs","commands.rs","events.rs","gift_card.rs","http.rs","main.rs","order.rs"]};
sourcesIndex["watcher"] = {"name":"","dirs":[{"name":"ui","files":["util.rs"]}],"files":["app.rs","main.rs","ui.rs"]};
createSourceSidebar();
