use actix_web::{App, HttpServer};

use chekov::prelude::*;
use chekov::Application;
use dotenv::dotenv;
use event_store::prelude::*;

use actix::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Default)]
struct Application1 {}
#[derive(Default)]
struct Application2 {}

impl chekov::Application for Application1 {
    type Storage = PostgresBackend;
}

impl chekov::Application for Application2 {
    type Storage = InMemoryBackend;
}

#[derive(Serialize)]
enum AccountStatus {
    Initialized,
    Active,
}

#[derive(Serialize)]
struct Account {
    account_id: Option<Uuid>,
    status: AccountStatus,
}

impl std::default::Default for Account {
    fn default() -> Self {
        Self {
            account_id: None,
            status: AccountStatus::Initialized,
        }
    }
}

impl Aggregate for Account {
    fn identity() -> &'static str {
        "account"
    }
}

impl EventApplier<AccountOpened> for Account {
    fn apply(&mut self, event: &AccountOpened) -> Result<(), ApplyError> {
        self.account_id = Some(event.account_id);
        self.status = AccountStatus::Active;

        Ok(())
    }
}
impl CommandExecutor<OpenAccount> for Account {
    fn execute(cmd: OpenAccount, state: &Self) -> Result<Vec<AccountOpened>, CommandExecutorError> {
        match state.status {
            AccountStatus::Initialized => Ok(vec![AccountOpened {
                account_id: cmd.account_id,
                name: cmd.name,
            }]),
            _ => Err(CommandExecutorError::Any),
        }
    }
}

#[derive(Clone, Debug, chekov::macros::Command, Serialize, Deserialize)]
#[command(event = "AccountOpened", aggregate = "Account")]
struct OpenAccount {
    #[command(identifier)]
    account_id: Uuid,
    name: String,
}

#[derive(Clone, Message, Debug, chekov::macros::Event, Deserialize, Serialize)]
#[rtype(result = "()")]
struct AccountOpened {
    account_id: Uuid,
    name: String,
}

struct AccountProjector {}

impl EventHandler for AccountProjector {
    fn started<A: Application>(
        &mut self,
        ctx: &mut actix::Context<chekov::event_handler::EventHandlerInstance<A, Self>>,
    ) {
        self.listen::<_, AccountOpened>(ctx);
    }
}

impl chekov::event_handler::Listening for AccountProjector {}

#[async_trait::async_trait]
impl chekov::event::Handler<AccountOpened> for AccountProjector {
    async fn handle(&self, _event: &AccountOpened) -> Result<(), ()> {
        // let _result = Account::create(event, &self.pool).await;
        Ok(())
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init();
    println!("Building PG pool.");
    let conn_str =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");

    let _ = Application1 {};
    let _ = Application2 {};
    let _ = PostgresBackend::with_url(&conn_str).await.unwrap();
    let _ = InMemoryBackend::default();

    let cmd = OpenAccount {
        account_id: Uuid::new_v4(),
        name: "".into(),
    };

    AccountProjector {}
        .builder()
        .register::<Application1>()
        .await;

    let _ = Router::<Application1>::dispatch::<_>(cmd.clone(), CommandMetadatas::default()).await;
    let _ = Router::<Application2>::dispatch::<_>(cmd, CommandMetadatas::default()).await;

    HttpServer::new(move || App::new())
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
