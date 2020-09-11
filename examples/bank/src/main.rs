use chekov::Aggregate;
use chekov::CommandExecutor;
use chekov::EventApplier;
use event_store::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

mod user;

enum AccountStatus {
    Initialized,
    Active,
    Deleted,
}

struct Account {
    _account_id: Option<Uuid>,
    status: AccountStatus,
}

impl std::default::Default for Account {
    fn default() -> Self {
        Self {
            _account_id: None,
            status: AccountStatus::Initialized,
        }
    }
}
impl Aggregate for Account {
    fn identity() -> &'static str {
        "account"
    }
}

impl CommandExecutor<DeleteAccount> for Account {
    fn execute(
        cmd: DeleteAccount,
        _state: &Self,
    ) -> Result<Vec<AccountDeleted>, chekov::CommandExecutorError> {
        Ok(vec![AccountDeleted {
            account_id: cmd.account_id,
        }])
    }
}

impl CommandExecutor<OpenAccount> for Account {
    fn execute(
        cmd: OpenAccount,
        state: &Self,
    ) -> Result<Vec<AccountOpened>, chekov::CommandExecutorError> {
        match state.status {
            AccountStatus::Initialized => Ok(vec![AccountOpened {
                account_id: cmd.account_id,
                name: cmd.name,
            }]),
            _ => Err(chekov::CommandExecutorError::Any),
        }
    }
}

impl EventApplier<AccountOpened> for Account {
    fn apply(&mut self, event: &AccountOpened) -> Result<(), chekov::ApplyError> {
        self._account_id = Some(event.account_id);
        self.status = AccountStatus::Active;

        Ok(())
    }
}

impl EventApplier<AccountDeleted> for Account {
    fn apply(&mut self, _event: &AccountDeleted) -> Result<(), chekov::ApplyError> {
        self.status = AccountStatus::Deleted;

        Ok(())
    }
}

#[derive(Debug)]
struct DeleteAccount {
    account_id: Uuid,
}

#[async_trait::async_trait]
impl chekov::Command for DeleteAccount {
    type Event = AccountDeleted;
    type Executor = Account;
    type ExecutorRegistry = ::chekov::AggregateInstanceRegistry<Account>;

    fn identifier(&self) -> ::std::string::String {
        self.account_id.to_string()
    }
    async fn dispatch(&self) -> Result<(), ()> {
        Ok(())
    }
}

#[derive(Clone, Debug, chekov::macros::Command, Serialize, Deserialize)]
#[command(event = "AccountOpened", aggregate = "Account")]
struct OpenAccount {
    #[command(identifier)]
    account_id: Uuid,
    name: String,
}

#[derive(chekov::macros::Event, Deserialize, Serialize)]
#[event(event_type = "MoneyMovement")]
enum MoneyMovementEvent {
    Deposited {
        account_id: Uuid,
    },
    Withdrawn {
        account_id: Uuid,
    },
    #[event(event_type = "MoneyDeleted")]
    Removed,
    Added(String),
}

#[derive(Debug, chekov::macros::Event, Deserialize, Serialize)]
struct AccountDeleted {
    account_id: Uuid,
}

#[derive(Debug, chekov::macros::Event, Deserialize, Serialize)]
struct AccountOpened {
    account_id: Uuid,
    name: String,
}

#[derive(chekov::macros::Event, Debug, Deserialize, Serialize)]
#[event(event_type = "Elixir.Conduit.Accounts.Events.UserRegistered")]
struct UserRegistered {
    email: String,
    hashed_password: String,
    user_uuid: Uuid,
    username: String,
}

type PGChekov = chekov::Chekov<PostgresBackend>;

use actix_web::{delete, get, post, put, Responder};
use actix_web::{middleware, web, App, HttpResponse, HttpServer};

#[get("/accounts")]
async fn find_all(_chekov: web::Data<std::sync::Arc<PGChekov>>) -> impl Responder {
    HttpResponse::InternalServerError().body("Unimplemented")
}

#[get("/accounts/{id}")]
async fn find(_id: web::Path<i32>, _chekov: web::Data<std::sync::Arc<PGChekov>>) -> impl Responder {
    HttpResponse::InternalServerError().body("Unimplemented")
}

#[post("/accounts")]
async fn create(
    account: web::Json<OpenAccount>,
    chekov: web::Data<std::sync::Arc<PGChekov>>,
) -> impl Responder {
    match chekov.dispatch(account.clone()).await {
        Ok(res) => HttpResponse::Ok().json(res), // <- send response
        Err(e) => HttpResponse::Ok().json(e),    // <- send response
    }
}

#[put("/accounts/{id}")]
async fn update(
    _id: web::Path<i32>,
    _account: web::Json<OpenAccount>,
    _chekov: web::Data<std::sync::Arc<PGChekov>>,
) -> impl Responder {
    HttpResponse::InternalServerError().body("Unimplemented")
}

#[delete("/accounts/{id}")]
async fn delete(
    id: web::Path<uuid::Uuid>,
    chekov: web::Data<std::sync::Arc<PGChekov>>,
) -> impl Responder {
    match chekov.dispatch(DeleteAccount { account_id: id.0 }).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::Ok().json(e),
    }
}

pub fn init<S: event_store::prelude::Storage>(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all);
    cfg.service(find);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
}
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    // Configure the storage (PG, InMemory,...)
    let storage = PostgresBackend::with_url("postgresql://postgres:postgres@localhost/event_store")
        .await
        .unwrap();

    let app = ::chekov::Chekov::with_storage(storage).await;

    let chekov = std::sync::Arc::new(app);
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(chekov.clone())
            .data(web::JsonConfig::default().limit(4096))
            .configure(init::<PostgresBackend>)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

// async fn main_event() -> Result<(), Box<dyn std::error::Error>> {
//     env_logger::init();
//     // Configure the storage (PG, InMemory,...)
//     let storage = PostgresBackend::with_url("postgresql://postgres:postgres@localhost/event_store")
//         .await
//         .unwrap();

//     let event_store = EventStore::builder()
//         .storage(storage)
//         .build()
//         .await
//         .unwrap();

//     let account_opened = AccountOpened {
//         account_id: Uuid::new_v4(),
//     };

//     let money_deposited = MoneyMovementEvent::MoneyDeposited {
//         account_id: Uuid::new_v4(),
//     };

//     let uuid = Uuid::new_v4().to_string();

//     let result = event_store::append()
//         .events(&[&account_opened])?
//         .event(&money_deposited)?
//         .to(&uuid)?
//         .expected_version(ExpectedVersion::AnyVersion)
//         .execute(&event_store)
//         .await;

//     println!("{:?}", result);

//     let result = event_store::read()
//         .stream(&uuid)?
//         .from(ReadVersion::Origin)
//         .limit(10)
//         .execute(&event_store)
//         .await?;

//     let _ = result
//         .first()
//         .unwrap()
//         .try_deserialize::<AccountOpened>()
//         .unwrap();

//     let mut stream = event_store::read()
//         .stream(&uuid)?
//         .from(ReadVersion::Origin)
//         .limit(2)
//         .into_stream()
//         .chunk_by(1)
//         .execute(&event_store)
//         .await;

//     // while let Some(notification) = stream.try_next().await? {
//     while let Some(notification) = stream.try_next().await? {
//         println!("[from stream]: {:?}", notification);
//     }

//     Ok(())
// }
