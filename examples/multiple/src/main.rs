use actix_web::{App, HttpServer};

use chekov::prelude::*;
use chekov::Application;
use dotenv::dotenv;
use event_store::prelude::*;

use lazy_static::lazy_static;

#[derive(Default)]
struct Main {}

impl chekov::Application for Main {
    type Storage = PostgresBackend;
}

struct AccountProjector {}

impl EventHandler for AccountProjector {}

impl chekov::event_handler::Listening for AccountProjector {}

lazy_static! {
    static ref PG_URL: String =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init();

    // let conn_str: String =
    //     std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");

    // let storage = chekov::application::PgStorageConfig::with_url(&conn_str);
    let storage = PostgresBackend::with_url(&PG_URL);

    Main::with_default()
        .storage(storage)
        .event_handler(AccountProjector {})
        .launch()
        .await;

    HttpServer::new(move || App::new())
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
