<h1 align="center">Chekov</h1>

<div align="center">
  A <code>CQRS/ES</code> framework for building application in <strong>Rust</strong>
</div>

<br />

<div align="center">
  
  [![Actions Status](https://github.com/freyskeyd/chekov/workflows/CI/badge.svg)](https://github.com/Freyskeyd/chekov/actions) [![Coverage Status](https://coveralls.io/repos/github/Freyskeyd/chekov/badge.svg?branch=master&service=github)](https://coveralls.io/github/Freyskeyd/chekov?branch=master) [![dependency status](https://deps.rs/repo/github/freyskeyd/chekov/status.svg)](https://deps.rs/repo/github/freyskeyd/chekov) [![Crates.io](https://img.shields.io/crates/v/chekov.svg)](https://crates.io/crates/chekov) [![doc.rs](https://docs.rs/chekov/badge.svg)](https://docs.rs/chekov) [![doc-latest](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://freyskeyd.github.io/chekov/chekov/)

</div>

## Table of Contents
- [Features](#features)
- [Getting started](#getting-started)
- [FAQ](#faq)
- [Need help?](#need-help)
- [Contributing](#contributing)

---

## Features

- `Postgres` EventStore backend
- Dispatch `Command` to `Aggregate` or `CommandHandler`
- Generate `Event` from `Aggregate` and persist them
- Apply `Event` to `Aggregate`
- Store and notify `Event` with subscriptions
- Dispatch `Event` to `EventHandler`

## Getting started

### Choosing an EventStore backend

Chekov works only with `Postgres` backend for now (and `InMemory` for test purpose). The choice is easy to make!

But some more backends need to be implemented, [see the related issue](#14).

### Defining Aggregates

An Aggregate is a struct that hold a domain state. Here's an example of a UserAggregate:

```rust
#[derive(Default, Aggregate)]
#[aggregate(identity = "user")]
struct User {
    user_id: Option<Uuid>,
    account_id: Option<Uuid>,
}

/// Define an Executor for the `CreateUser` command
/// The result is a list of events in case of success
impl CommandExecutor<CreateUser> for User {
  fn execute(cmd: CreateUser, _state: &Self) -> Result<Vec<UserCreated>, CommandExecutorError> {
    Ok(vec![UserCreated {
      user_id: cmd.user_id,
      account_id: cmd.account_id,
    }])
  }
}

/// Define an Applier for the `UserCreated` event
/// Applier is a mutation action on the aggregate
#[chekov::applier]
impl EventApplier<UserCreated> for User {
  fn apply(&mut self, event: &UserCreated) -> Result<(), ApplyError> {
    self.user_id = Some(event.user_id);
    self.account_id = Some(event.account_id);

    Ok(())
  }
}

```

### Defining Commands

You need to create a struct per command, any type of struct can implement `Command` but we advise to use struct for a better readability.

A command can only produce (or not) one type of events and it targets a single Aggregate.
A command must have a single and unique `identifier` that is used to route the command to the right target.

```rust

#[derive(Debug, Command)]
#[command(event = "UserCreated", aggregate = "User")]
struct CreateUser {
    #[command(identifier)]
    user_id: Uuid,
    account_id: Uuid,
}
```

### Defining Events

An `Event` can be a `struct` or an `enum`.

```rust
#[derive(Event, Deserialize, Serialize)]
struct UserCreated {
    user_id: Uuid,
    account_id: Uuid,
}
```

### Defining Saga

Not implemented yet

## FAQ

### Does `Chekov` is production ready ?

No its not. Some critical part of the project are still not implemented and a lot of code needs to be refactored before that.

## Need Help?

Feel free to open issue in case of bugs or features requests. [Discussions](https://github.com/Freyskeyd/chekov/discussions) are also a great starts if you have issue that are not bugs nor features requests.

## Contributing

The project is really early staged and have a lot of pending tasks, one major tasks is to produce a roadmap or some issues that can be used to expose the project vision. Feel free to open a [Discussions](https://github.com/Freyskeyd/chekov/discussions) around it if you want !
