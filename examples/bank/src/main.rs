use actix_web::{middleware, web, App, HttpServer};
use chekov::{application::DefaultEventResolver, prelude::*};
use event_store::prelude::*;
use sqlx::PgPool;

mod account;
mod commands;
mod events;
mod http;

use events::*;

pub fn init<S: event_store::prelude::Storage>(cfg: &mut web::ServiceConfig) {
    cfg.service(http::find_all);
    cfg.service(http::find);
    cfg.service(http::create);
    cfg.service(http::update);
    cfg.service(http::delete);
}

#[derive(Default)]
struct DefaultApp {}

impl chekov::Application for DefaultApp {
    type Storage = PostgresBackend;
    type EventResolver = DefaultEventResolver<Self>;
}

#[allow(dead_code)]
fn configure_events<A: chekov::Application>() -> DefaultEventResolver<A> {
    DefaultEventResolver::default()
        .register::<AccountOpened>()
        .register::<AccountUpdated>()
        .register::<AccountDeleted>()
        .register::<MoneyMovementEvent>()
        .register::<UserRegistered>()
}

#[actix::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let db_pool = PgPool::connect("postgresql://postgres:postgres@localhost/bank")
        .await
        .unwrap();

    // Configure the storage (PG, InMemory,...)
    DefaultApp::with_default()
        .event_handler(account::AccountProjector {
            pool: db_pool.clone(),
        })
        .storage(PostgresBackend::with_url(
            "postgresql://postgres:postgres@localhost/event_store_bank",
        ))
        .event_resolver(configure_events())
        .launch()
        .await;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(db_pool.clone())
            .data(web::JsonConfig::default().limit(4096))
            .configure(init::<PostgresBackend>)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
