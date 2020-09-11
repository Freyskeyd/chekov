use actix::prelude::*;
pub use chekov_macros as macros;
pub use event_store;
use event_store::prelude::*;
pub use event_store::Event;
use log::trace;
use std::convert::TryFrom;

#[derive(Default)]
pub struct AggregateInstance<A: Aggregate> {
    inner: A,
}

impl<A: Aggregate> ::actix::Actor for AggregateInstance<A> {
    type Context = ::actix::Context<Self>;
}

impl<C: Command, S: event_store::prelude::Storage> ::actix::Handler<Dispatch<C, S>>
    for AggregateInstance<C::Executor>
{
    type Result = Result<Vec<C::Event>, CommandExecutorError>;
    fn handle(&mut self, cmd: Dispatch<C, S>, _ctx: &mut Self::Context) -> Self::Result {
        trace!(
            "Executing command {:?} from {} {:?}",
            std::any::type_name::<C>(),
            std::any::type_name::<Self>(),
            cmd.command
        );
        C::Executor::execute(cmd.command, &self.inner)
    }
}
#[derive(Default)]
pub struct AggregateInstanceRegistry<A: Aggregate> {
    registry: ::std::collections::HashMap<String, ::actix::Addr<AggregateInstance<A>>>,
}

impl<A: Aggregate> ::actix::registry::ArbiterService for AggregateInstanceRegistry<A> {}
impl<A: Aggregate> ::actix::Supervised for AggregateInstanceRegistry<A> {}
impl<A: Aggregate> ::actix::Actor for AggregateInstanceRegistry<A> {
    type Context = ::actix::Context<Self>;
}

use actix_interop::{critical_section, with_ctx, FutureInterop};

impl<C: Command, S: event_store::prelude::Storage> ::actix::Handler<Dispatch<C, S>>
    for AggregateInstanceRegistry<C::Executor>
{
    type Result = actix::ResponseActFuture<Self, Result<Vec<C::Event>, CommandExecutorError>>;

    fn handle(&mut self, cmd: Dispatch<C, S>, _ctx: &mut Self::Context) -> Self::Result {
        // Open aggregate
        trace!(
            "Dispatching {:?} from {}",
            std::any::type_name::<C>(),
            std::any::type_name::<Self>()
        );
        async move {
            critical_section::<Self, _>(async {
                let id = cmd.command.identifier();
                let addr: Addr<_> = if let Some(addr) =
                    with_ctx(|actor: &mut Self, _| actor.registry.get(&id).cloned())
                {
                    trace!(
                        "{}({}) already started",
                        std::any::type_name::<C::Executor>(),
                        id
                    );
                    addr
                } else {
                    // start it?
                    trace!("{}({}) not found", std::any::type_name::<C::Executor>(), id);
                    trace!(
                        "Rebuilding state for {}({}) ",
                        std::any::type_name::<C::Executor>(),
                        id
                    );
                    let event_store: event_store::EventStore<S> =
                        ::actix::Arbiter::get_item::<event_store::EventStore<S>, _, EventStore<S>>(
                            |event_store| event_store.duplicate(),
                        );
                    let result = match event_store::read()
                        .stream(&id)
                        .unwrap()
                        .from(ReadVersion::Origin)
                        .limit(10)
                        .execute(&event_store)
                        .await
                    {
                        Ok(events) => events,
                        Err(_) => panic!(""),
                    };

                    AggregateInstance::create(move |ctx_agg| {
                        trace!("Creating aggregate instance");
                        let _ctx_address = ctx_agg.address();
                        let mut inner = C::Executor::default();
                        for event in result {
                            let _res = match C::Event::try_from(event.clone()) {
                                Ok(parsed_event) => inner.apply(&parsed_event).map_err(|_| ()),
                                _ => {
                                    // println!(
                                    //     "ERROR PARSING EVENT for {:?} on {:?}",
                                    //     self.instance, msg
                                    // );
                                    Err(())
                                }
                            };
                        }
                        AggregateInstance {
                            // subscribtion: Subscriber::create(move |ctx| Subscriber {
                            //     stream: "account".to_string(),
                            //     aggr: ctx_address,
                            // }),
                            inner,
                        }

                        // let fut = Registry::from_registry()
                        //     .send(Register {
                        //         stream: "account".to_string(),
                        //         addr: ctx_agg.address(),
                        //     })
                        // .into_actor(&instance)
                        //     .map(|_, _, _| ());

                        // ctx_agg.wait(fut);
                        // instance
                    })
                };

                match addr.send(cmd).await {
                    Ok(res) => {
                        if let Ok(ref events) = res {
                            trace!("Generated {:?}", events.len());

                            let event_store: event_store::EventStore<S> =
                                ::actix::Arbiter::get_item::<
                                    event_store::EventStore<S>,
                                    _,
                                    EventStore<S>,
                                >(|event_store| {
                                    event_store.duplicate()
                                });

                            let ev: Vec<&_> = events.iter().collect();
                            let _result = event_store::append()
                                .events(&ev[..])
                                .unwrap()
                                .to(&id)
                                .unwrap()
                                .expected_version(event_store::prelude::ExpectedVersion::AnyVersion)
                                .execute(&event_store)
                                .await;
                        }
                        res
                    }
                    Err(_) => Err(CommandExecutorError::Any),
                }
            })
            .await
        }
        .interop_actor_boxed(self)
    }
}

pub struct Chekov<B: event_store::prelude::Storage> {
    pub _event_store: event_store::EventStore<B>,
    _router: Addr<Router<B>>,
}

impl<B> Chekov<B>
where
    B: event_store::prelude::Storage,
{
    pub async fn dispatch<C: Command>(&self, cmd: C) -> Result<Vec<C::Event>, CommandExecutorError>
    where
        <C as Command>::ExecutorRegistry: actix::Handler<Dispatch<C, B>>,
    {
        // Router::<B>::dispatch(cmd).await

        self._router
            .send(Dispatch::<C, B> {
                storage: std::marker::PhantomData,
                command: cmd,
            })
            .await?
    }

    pub async fn with_storage(storage: B) -> Self {
        trace!(
            "Creating a new Chekov instance with {}",
            std::any::type_name::<B>()
        );
        let event_store = EventStore::builder()
            .storage(storage)
            .build()
            .await
            .unwrap();

        ::actix::Arbiter::set_item(event_store.duplicate());
        let router = Router {
            _event_store: event_store.duplicate(),
            _before_dispatch: vec![],
        };
        let addr = router.start();

        ::actix::Registry::set(addr.clone());
        Self {
            _event_store: event_store,
            _router: addr,
        }
    }
}

#[derive(serde::Serialize, Debug)]
pub enum CommandExecutorError {
    Any,
}
impl std::convert::From<actix::MailboxError> for CommandExecutorError {
    fn from(_: actix::MailboxError) -> Self {
        Self::Any
    }
}
#[derive(Debug)]
pub enum ApplyError {
    Any,
}

#[async_trait::async_trait]
pub trait Command: std::fmt::Debug + Send + 'static {
    type Event: Event;
    type Executor: CommandExecutor<Self> + EventApplier<Self::Event>;
    type ExecutorRegistry: ArbiterService;

    fn identifier(&self) -> String;
    async fn dispatch(&self) -> Result<(), ()>;
}

pub trait Aggregate: Default + std::marker::Unpin + 'static {
    fn identity() -> &'static str;
}

pub trait CommandExecutor<T: Command + ?Sized>: Aggregate {
    fn execute(cmd: T, state: &Self) -> Result<Vec<T::Event>, CommandExecutorError>;
}

pub trait EventApplier<E: Event> {
    fn apply(&mut self, event: &E) -> Result<(), ApplyError>;
}

pub trait CommandHandler<C: Command + ?Sized> {
    fn execute(command: C, executor: &C::Executor) -> Result<Vec<C::Event>, CommandExecutorError>;
}

#[derive(Default, Debug)]
pub struct Registry {
    registry: std::collections::HashMap<String, Vec<actix::Recipient<EventEnvelope>>>,
}

pub struct Router<S: event_store::prelude::Storage> {
    _event_store: event_store::EventStore<S>,
    _before_dispatch: Vec<String>,
}

impl<S: event_store::prelude::Storage> std::default::Default for Router<S> {
    fn default() -> Self {
        unimplemented!()
    }
}
impl<S: event_store::prelude::Storage> ::actix::Actor for Router<S> {
    type Context = ::actix::Context<Self>;
}

impl<S: event_store::prelude::Storage> ::actix::registry::ArbiterService for Router<S> {}
impl<S: event_store::prelude::Storage> ::actix::Supervised for Router<S> {}

#[async_trait::async_trait]
impl<S: event_store::prelude::Storage, C: Command> Dispatchable<C, S> for Router<S> {
    async fn dispatch(&self, cmd: C) -> Result<Vec<C::Event>, CommandExecutorError>
    where
        <C as Command>::ExecutorRegistry: actix::Handler<Dispatch<C, S>>,
    {
        Self::from_registry()
            .send(Dispatch::<C, S> {
                storage: std::marker::PhantomData,
                // to: <C::ExecutorRegistry as ArbiterService>::from_registry()
                //     .recipient::<Dispatch<C, S>>(),
                command: cmd,
            })
            .await?
        // // Execute before_dispatch middleware
        // // Open aggregate
        // // Create ExecutionContext
        // // Execute after_dispatch middleware
    }
}
impl<S: event_store::prelude::Storage, T: Command> ::actix::Handler<Dispatch<T, S>> for Router<S>
where
    <T as Command>::ExecutorRegistry: actix::Handler<Dispatch<T, S>>,
{
    type Result = actix::ResponseActFuture<Self, Result<Vec<T::Event>, CommandExecutorError>>;

    fn handle(&mut self, msg: Dispatch<T, S>, _ctx: &mut Self::Context) -> Self::Result {
        // let to = msg.to.clone();

        let to =
            <T::ExecutorRegistry as ArbiterService>::from_registry().recipient::<Dispatch<T, S>>();
        Box::pin(async move { to.send(msg).await? }.into_actor(self))
    }
}

#[async_trait::async_trait]
pub trait Dispatchable<C, S>
where
    C: Command,
    S: event_store::prelude::Storage,
{
    async fn dispatch(&self, cmd: C) -> Result<Vec<C::Event>, CommandExecutorError>
    where
        <C as Command>::ExecutorRegistry: actix::Handler<Dispatch<C, S>>;
}

impl actix::registry::ArbiterService for Registry {}
impl actix::Supervised for Registry {}
impl actix::Actor for Registry {
    type Context = actix::Context<Self>;
}

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<Vec<T::Event>, CommandExecutorError>")]
pub struct Dispatch<T: Command, S: event_store::prelude::Storage> {
    pub storage: std::marker::PhantomData<S>,
    // pub to: actix::Recipient<Dispatch<T, S>>,
    pub command: T,
}

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<usize, ()>")]
pub struct EventEnvelope(RecordedEvent);

// impl<T> actix::Handler<Dispatch<T>> for Router
// where
//     T: Command,
// {
//     type Result = actix::ResponseActFuture<Self, Result<usize, ()>>;

//     fn handle(&mut self, msg: Dispatch<T>, _: &mut actix::Context<Self>) -> Self::Result {
//         let agg_identity = format!(
//             "{}-{}",
//             T::Executor::identity(),
//             msg.command.identifier().to_string()
//         );

//         println!("{:#?}", agg_identity);

//         // let v: Vec<actix::Recipient<EventEnvelope>> = Vec::new();
//         // let futs = self
//         //     .event_registry
//         //     .get(&msg.0.stream_uuid)
//         //     .get_or_insert_with(|| &v)
//         //     .iter()
//         //     .map(|r| r.send(msg.clone()))
//         //     .collect::<Vec<RecipientRequest<EventEnvelope>>>();

//         Box::new(async { 1 }.into_actor(self).map(|res, _, _| {
//             // let res = res
//             //     .into_iter()
//             //     .map(Result::unwrap)
//             //     .map(Result::unwrap)
//             //     .sum::<usize>();

//             Ok(res)
//         }))
//     }
// }

impl actix::Handler<EventEnvelope> for Registry {
    type Result = actix::ResponseActFuture<Self, Result<usize, ()>>;

    fn handle(&mut self, msg: EventEnvelope, _: &mut actix::Context<Self>) -> Self::Result {
        let v: Vec<actix::Recipient<EventEnvelope>> = Vec::new();
        let futs = self
            .registry
            .get(&msg.0.stream_uuid)
            .get_or_insert_with(|| &v)
            .iter()
            .map(|r| r.send(msg.clone()))
            .collect::<Vec<RecipientRequest<EventEnvelope>>>();

        Box::pin(
            async { futures::future::join_all(futs).await }
                .into_actor(self)
                .map(|res, _, _| {
                    let res = res
                        .into_iter()
                        .map(Result::unwrap)
                        .map(Result::unwrap)
                        .sum::<usize>();

                    Ok(res)
                }),
        )
    }
}

// pub struct DummyCommandHandler {}

// impl<T> CommandHandler<T> for DummyCommandHandler
// where
//     T: Command,
// {
//     fn execute(command: T, executor: &T::Executor) -> Result<Vec<T::Event>, CommandExecutorError> {
//         executor.execute(command)
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    // use event_store::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug)]
    struct MyCommand(pub String);

    #[async_trait::async_trait]
    impl Command for MyCommand {
        type Event = MyEvent;
        type Executor = AggTest;
        type ExecutorRegistry = crate::AggregateInstanceRegistry<AggTest>;
        fn identifier(&self) -> String {
            self.0.clone()
        }
        async fn dispatch(&self) -> Result<(), ()> {
            Ok(())
        }
    }

    // struct MyEventRegistry {}

    // struct MyCmdHandler {}

    // impl CommandHandler<MyCommand> for MyCmdHandler {
    //     fn execute(
    //         command: MyCommand,
    //         executor: &<MyCommand as Command>::Executor,
    //     ) -> Result<Vec<<MyCommand as Command>::Event>, CommandExecutorError> {
    //         if command.0.is_empty() {
    //             return Err(CommandExecutorError::Any);
    //         }
    //         executor.execute(command)
    //     }
    // }

    #[derive(Serialize, Deserialize)]
    struct MyEvent(String);
    impl Event for MyEvent {
        fn event_type(&self) -> &'static str {
            "MyEvent"
        }
    }

    impl std::convert::TryFrom<event_store::prelude::RecordedEvent> for MyEvent {
        type Error = ();
        fn try_from(e: event_store::prelude::RecordedEvent) -> Result<Self, Self::Error> {
            serde_json::from_value(e.data).map_err(|_| ())
        }
    }

    #[derive(Default)]
    struct AggTest {
        handled_event: Option<String>,
    }
    impl Aggregate for AggTest {
        fn identity() -> &'static str {
            "agg_test"
        }
    }

    impl CommandExecutor<MyCommand> for AggTest {
        fn execute(
            cmd: MyCommand,
            _state: &Self,
        ) -> Result<Vec<<MyCommand as Command>::Event>, CommandExecutorError> {
            Ok(vec![MyEvent(cmd.0)])
        }
    }

    impl EventApplier<MyEvent> for AggTest {
        fn apply(&mut self, event: &MyEvent) -> Result<(), ApplyError> {
            self.handled_event = Some(event.0.clone());
            Ok(())
        }
    }

    #[test]
    fn test_command_executor() {
        let _cmd = MyCommand("a command".into());

        let _reg = Registry::default();

        let _agg = AggTest {
            handled_event: None,
        };
    }
}
