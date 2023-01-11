use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tracing::Subscriber;
use tracing_core::span;
use tracing_subscriber::field::display::Messages;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

use chekov_api as proto;

mod builder;
mod command;
mod server;
mod visitors;

use crate::visitors::AggregateInstanceData;
use builder::ConsoleLayerBuilder;
use command::Command;
use command::Watch;
use server::Server;

#[derive(Debug)]
pub struct ConsoleLayer {
    tx: mpsc::Sender<ConsoleEvent>,
}

impl ConsoleLayer {
    pub fn builder() -> ConsoleLayerBuilder {
        ConsoleLayerBuilder::default()
    }

    pub fn build(builder: ConsoleLayerBuilder) -> (Self, Server) {
        let (tx, events) = mpsc::channel(1024 * 100);
        let (subscribe, rpcs) = mpsc::channel(256);

        let aggregator = Aggregator::new(events, rpcs);

        let layer = Self { tx };

        let server = Server {
            addr: builder.server_addr,
            client_buffer: Server::DEFAULT_CLIENT_BUFFER_CAPACITY,
            aggregator: Some(aggregator),
            subscribe,
        };

        (layer, server)
    }
}

impl<S> Layer<S> for ConsoleLayer
where
    S: Subscriber + std::fmt::Debug + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = attrs.metadata();

        match (metadata.name(), metadata.target()) {
            (_, "chekov::event::handler") => {
                // println!("{:#?}", event);
            }
            (_, target) if target.starts_with("chekov::aggregate::instance") => {
                let span = ctx.span(id).expect("in new_span but span does not exist");
                println!(
                    "on_new_span : {:?}",
                    span.extensions().get::<AggregateInstanceData>()
                );
                if span.extensions().get::<AggregateInstanceData>().is_none() {
                    let data = AggregateInstanceData::new(attrs);
                    span.extensions_mut().insert(data);

                    let ext = span.extensions();
                    let data = ext
                        .get::<AggregateInstanceData>()
                        .expect("AggregateInstanceData cannot be found in extensions");
                    println!("{:?}", data);
                }
            }
            _ => {}
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let metadata = event.metadata();

        match (metadata.name(), metadata.target()) {
            (_, "chekov::event::handler") => {
                // println!("{:#?}", event);
            }
            (_, target) if target.starts_with("chekov::aggregate::instance") => {
                if let Some(span) = ctx.event_span(event) {
                    if let Some(data) = span.extensions().get::<AggregateInstanceData>() {
                        println!("{:?}", data);
                    }
                }
            }
            _ => {}
        }
    }

    fn on_enter(&self, _id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {}
}

#[derive(Debug)]
enum ConsoleEvent {
    AggregateUpdate,
}

#[derive(Debug)]
pub(crate) struct Aggregator {
    events: mpsc::Receiver<ConsoleEvent>,
    rpcs: mpsc::Receiver<Command>,

    /// Currently active RPCs streaming task events.
    watchers: Vec<Watch<proto::client::Update>>,
}

impl Aggregator {
    fn add_update_subscription(&mut self, subscription: Watch<proto::client::Update>) {
        tracing::debug!("new instrument subscription");

        let update = &proto::client::Update {
            now: Some(SystemTime::now().into()),
            stream_update: None,
            aggregate_update: None,
            event_update: None,
        };

        // Send the initial state --- if this fails, the subscription is already dead
        if subscription.update(update) {
            self.watchers.push(subscription)
        }
    }

    pub(crate) async fn run(mut self) {
        let mut publish = tokio::time::interval(Duration::from_secs(1));
        loop {
            let should_send = tokio::select! {
                // if the flush interval elapses, flush data to the client
                _ = publish.tick() => {
                    true
                }

                // a new command from a client
                cmd = self.rpcs.recv() => {
                    match cmd {
                        Some(Command::Update(subscription)) => {
                            self.add_update_subscription(subscription);
                        },

                        _ => {
                            tracing::debug!("rpc channel closed, terminating");
                            return;
                        }
                    };

                    false
                }
            };

            if !self.watchers.is_empty() && should_send {
                self.publish();
            }
        }
    }

    fn publish(&mut self) {
        let update = proto::client::Update {
            now: Some(SystemTime::now().into()),
            stream_update: None,
            aggregate_update: None,
            event_update: None,
        };

        self.watchers
            .retain(|watcher: &Watch<proto::Update>| watcher.update(&update));
    }

    fn new(events: mpsc::Receiver<ConsoleEvent>, rpcs: mpsc::Receiver<Command>) -> Aggregator {
        Self {
            watchers: Vec::new(),
            events,
            rpcs,
        }
    }
}

pub fn init() {
    tracing::info!("Initialize consoler layer");
    ConsoleLayer::builder().init();
}
