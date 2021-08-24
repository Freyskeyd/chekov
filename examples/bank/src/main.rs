use std::env;

use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpServer,
};
use chekov::{application::DefaultEventResolver, prelude::*};
use event_store::prelude::*;
use sqlx::PgPool;

mod account;
mod commands;
mod events;
mod http;

use events::*;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(http::find_all);
    cfg.service(http::find);
    // cfg.service(http::create);
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

    let _ = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(Data::new(db_pool.clone()))
            .app_data(web::JsonConfig::default().limit(4096))
            .configure(init)
    })
    .bind(env::var("RUNTIME_ENDPOINT").expect("Must define the RUNTIME_ENDPOINT"))?
    .run()
    .await?;

    tokio::signal::ctrl_c().await.unwrap();
    println!("Ctrl-C received, shutting down");
    Ok(())
}
