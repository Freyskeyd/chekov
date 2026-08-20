# Chekov 0.1 behavioral baseline

This document records how Chekov 0.1 behaves today. The 0.2 refactor needs to preserve each behavior or change it deliberately. FRE-32 covers the default workspace and in-memory execution paths, not live Postgres/Docker integration.

## Reproducing the baseline

The baseline was captured with:

- `rustc 1.97.1 (8bab26f4f 2026-07-14)`
- `cargo 1.97.1 (c980f4866 2026-06-30)`
- Darwin 25.5.0 arm64

Run from the workspace root:

```console
cargo check
cargo build
cargo test
```

`cargo test` uses the workspace `default-members` (`crates/*`). It compiles the Postgres crates and runs their database-independent unit tests, but it does not start Postgres or run a Docker-backed integration suite. The examples are also outside this command.

The Actix 0.12 test macros only compile with `actix_derive = 0.6.0`. Later 0.6.x releases expand through an `actix::__private` API that Actix 0.12 does not export, so the workspace pins the compatible macro crate explicitly.

At capture time, the suite reported 70 passing tests, one ignored test, and no failures. This count includes unit and documentation tests.

## Command and aggregate execution

- An `AggregateInstanceRegistry<A>` is an Actix `SystemService` with an in-process map from aggregate identifier to actor address.
- Dispatch reuses the actor registered for an identifier. If there is no actor, it reconstructs the aggregate from its stream, subscribes it to that stream, caches the address, and executes the command.
- A command executor receives a clone of the current aggregate state and returns either events or a `CommandExecutorError`.
- Events are applied to the cloned state before persistence. The actor replaces its live state and version only after the append succeeds.
- A successful command returns its emitted events. A command that emits no events succeeds with an empty vector.
- A failed command returns an error and leaves aggregate state and version unchanged. If it is the first command for an identifier, the stream remains nonexistent.
- A successful no-op command currently creates an empty stream. This differs from a failed first command and must be preserved or changed explicitly in 0.2.

Baseline tests:

- `tests::aggregates::runtime::can_execute_a_command`
- `tests::aggregates::runtime::can_recover_from_fail_execution`
- `tests::aggregates::persistency::failed_command_leaves_aggregate_and_stream_unchanged`
- `tests::aggregates::persistency::should_not_persist_events_when_command_returns_no_events`

## Persistence and versions

- Appends preserve event order.
- The aggregate sends its current version as `ExpectedVersion::Version(current_version)`.
- The in-memory appender creates a missing stream when that expected version permits creation.
- Stored stream versions and aggregate versions advance once per applied event.
- Multiple events from one command are appended through one appender request. The storage contract does not expose a transaction abstraction outside the actor runtime.
- Restarting an aggregate removes its cached actor. The next start replays canonical events from the event store.
- Replay is streamed in batches of 100 events; the 300-event restart test crosses this boundary.

Baseline tests:

- `tests::aggregates::persistency::should_persist_pending_events_in_order_applied`
- `tests::aggregates::persistency::should_reload_persisted_events_when_restarting_aggregate_process`
- `tests::aggregates::persistency::should_reload_persisted_events_in_batches_when_restarting_aggregate_process`
- `tests::aggregates::state::should_rebuild_his_state_from_previously_append_events`

## Replay and subscription delivery

- Starting an aggregate creates a transient PubSub subscription for its stream.
- A recorded event at the next stream version is resolved by event type, applied, and increments the aggregate version.
- An already-seen event is ignored when its stream version is at or below the aggregate version. Duplicate delivery is therefore idempotent for aggregate state.
- A future event with a version gap is an apply error. The direct `ResolveAndApply` handler stops the actor on that error.
- Unknown event types are ignored when no resolver is found, and the version does not advance. Chekov 0.2 should report this as a corruption error.
- Subscription notification handlers discard individual apply errors, unlike direct `ResolveAndApply`. This is not a durable projection guarantee.

Baseline tests:

- `tests::aggregates::subscription::aggregate_should_starts_a_pubsub_subscription`
- `tests::aggregates::subscription::should_ignore_already_seen_events`
- `tests::aggregates::subscription::should_stop_aggregate_process_when_unexpected_event_received`
- the `event_store::subscriptions::tests` suite

## Concurrency limits

Chekov 0.1 does not provide a cross-process concurrency guarantee. Its actor registries and addresses exist only inside one Actix system.

The aggregate command handler returns a `ResponseActFuture` without waiting on the actor context. Another command or event can be processed while command execution and persistence are in flight. Chekov therefore cannot rely on per-aggregate serialization, even within one process. The 0.2 design replaces this with atomic expected-version validation in storage.

There is no reliable concurrency regression test in 0.1. A timing-based characterization test would capture scheduler behavior rather than a supported invariant, so FRE-32 records the limitation instead.

## Known gaps and noisy checks

- `router::tests::can_route_a_command` is ignored and contains no behavioral assertion.
- `should_persist_event_metadata` is a passing placeholder with no implementation or assertion. Metadata persistence is not established.
- `should_notify_aggregate_and_mutate_its_state` does not publish an event; it only checks the initial state.
- No live Postgres behavior is exercised by the default test command.
- No test establishes safe concurrent command execution.
- The current compiler warnings cover unused imports and dead code, an irrefutable pattern, confusing lifetime syntax, and future incompatibility in `sqlx-core 0.6.3`. FRE-33 owns compiler and tooling modernization.

## Required 0.2 decisions

Chekov 0.2 intentionally changes these areas:

- Replace Actix actor lifecycle and routing with a synchronous deterministic domain boundary plus async orchestration.
- Replace in-process actor exclusivity with atomic expected-version storage semantics.
- Decide whether successful no-op commands create streams; specify the behavior in the new storage contract.
- Turn unknown event types, corrupt payloads, and replay gaps into explicit infrastructure errors.
- Persist event metadata deliberately, including correlation and causation identifiers.
- Separate durable subscription/projection checkpoints from transient PubSub delivery.

Chekov 0.2 should retain these invariants:

- Event order is stable.
- Replay rebuilds the same state, including across batch boundaries.
- Failed commands do not change canonical state, aggregate state, or version.
- Duplicate delivery does not apply an event twice.
- Successful state publication occurs only after persistence succeeds.
