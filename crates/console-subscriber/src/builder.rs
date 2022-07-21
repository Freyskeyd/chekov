use std::net::SocketAddr;
use std::thread;

use tokio::runtime;
use tracing::Subscriber;
use tracing_subscriber::fmt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter, layer::SubscriberExt, Layer};

use crate::{ConsoleLayer, Server};

pub struct ConsoleLayerBuilder {
    pub(crate) server_addr: SocketAddr,
    self_trace: bool,
}

impl ConsoleLayerBuilder {
    pub fn init(self) {
        let fmt_filter = std::env::var("RUST_LOG")
            .ok()
            .and_then(|log_filter| match log_filter.parse::<filter::EnvFilter>() {
                Ok(targets) => Some(targets),
                Err(e) => {
                    eprintln!(
                        "failed to parse filter environment variable `{}={:?}`: {}",
                        "RUST_LOG", log_filter, e
                    );
                    None
                }
            })
            .unwrap_or_else(|| {
                "error"
                    .parse::<filter::EnvFilter>()
                    .expect("`error` filter should always parse successfully")
            });

        let console_layer = self.spawn();
        tracing_subscriber::registry()
            .with(console_layer)
            // .with(filter_layer)
            .with(fmt::layer().with_filter(fmt_filter))
            .init();
    }

    pub fn enable_self_trace(self, self_trace: bool) -> Self {
        Self { self_trace, ..self }
    }

    fn spawn<S>(self) -> impl Layer<S>
    where
        S: Subscriber + std::fmt::Debug + for<'a> LookupSpan<'a>,
    {
        let self_trace = self.self_trace;

        let (layer, server) = self.build();

        thread::Builder::new()
            .name("console_subscriber".into())
            .spawn(move || {
                let _subscriber_guard;
                if !self_trace {
                    _subscriber_guard = tracing::subscriber::set_default(
                        tracing_core::subscriber::NoSubscriber::default(),
                    );
                }
                let runtime = runtime::Builder::new_current_thread()
                    .enable_io()
                    .enable_time()
                    .build()
                    .expect("console subscriber runtime initialization failed");

                runtime.block_on(async move {
                    server
                        .serve()
                        .await
                        .expect("console subscriber server failed")
                });
            })
            .expect("console subscriber could not spawn thread");
        layer
    }

    fn build(self) -> (ConsoleLayer, Server) {
        ConsoleLayer::build(self)
    }
}

impl Default for ConsoleLayerBuilder {
    fn default() -> Self {
        Self {
            server_addr: SocketAddr::new(Server::DEFAULT_IP, Server::DEFAULT_PORT),
            self_trace: false,
        }
    }
}
