use std::env;

use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpServer,
};
use chekov::prelude::*;
use sqlx::PgPool;

mod account;
mod commands;
mod events;
mod gift_card;
mod http;
mod order;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(http::create_account);
    cfg.service(http::create_gift_card);
    cfg.service(http::create_order);
    cfg.service(http::add_order_item);
}

#[derive(Default)]
struct DefaultApp {}

impl Application for DefaultApp {
    type Storage = PostgresBackend;
    type EventBus = PostgresEventBus;
}

#[actix::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let db_pool = PgPool::connect("postgresql://postgres:postgres@localhost/gift_shop")
        .await
        .unwrap();

    // Configure the storage (PG, InMemory,...)
    DefaultApp::with_default()
        .listener_url("postgresql://postgres:postgres@localhost/event_store_gift_shop".into())
        .event_handler(account::AccountProjector {
            pool: db_pool.clone(),
        })
        .event_handler(gift_card::GiftCardProjector {
            pool: db_pool.clone(),
        })
        .event_handler(order::OrderProjector {
            pool: db_pool.clone(),
        })
        .storage(PostgresBackend::with_url(
            "postgresql://postgres:postgres@localhost/event_store_gift_shop",
        ))
        .event_bus(PostgresEventBus::initiate(
            "postgresql://postgres:postgres@localhost/event_store_gift_shop".into(),
        ))
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
