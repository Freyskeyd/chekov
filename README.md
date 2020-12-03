<h1 align="center">Chekov</h1>

<div align="center">
  A <code>CQRS/ES</code> framework for building application in <strong>Rust</strong>
</div>

<br />

<div align="center">
  
  [![Actions Status](https://github.com/freyskeyd/chekov/workflows/CI/badge.svg)](https://github.com/Freyskeyd/chekov/actions) [![Coverage Status](https://coveralls.io/repos/github/Freyskeyd/chekov/badge.svg?branch=master&service=github)](https://coveralls.io/github/Freyskeyd/chekov?branch=master) [![dependency status](https://deps.rs/repo/github/freyskeyd/chekov/status.svg)](https://deps.rs/repo/github/freyskeyd/chekov) [![Crates.io](https://img.shields.io/crates/v/chekov.svg)](https://crates.io/crates/chekov) [![doc.rs](https://docs.rs/chekov/badge.svg)](https://docs.rs/chekov)

</div>

## Table of Contents
- [Features](#features)
- [Getting started](#getting-started)
- [Using Chekov](#using-chekov)
- [Deployment](#deployment)
- [FAQ](#faq)
- [Need help?](#need-help)
- [Contributing](#contributing)

---

## Features

- `Postgres` EventStore backend
- Dispatch `Command` to `Aggregate`

## Getting started

### Choosing an EventStore backend

Chekov works only with `Postgres` backend for now. The choice is easy to make!

But some more backends need to be implemented, [see the related issue](#).

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
fn apply_user_created(state: &mut User, event: &UserCreated) -> Result<(), ApplyError> {
  self.user_id = Some(event.user_id);
  self.account_id = Some(event.account_id);

  Ok(())
}

// Register this applier in chekov
chekov::macros::apply_event!(DefaultApp, User, UserCreated, apply_user_created);
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
