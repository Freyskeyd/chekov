use actix::prelude::*;
use actix_web::{delete, get, post, put};
use actix_web::{middleware, web, App, HttpServer};
use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use chekov::prelude::*;
use event_store::prelude::*;
use futures::future::{ready, Ready};
use log;
use serde::Deserialize;
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use std::any::TypeId;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize)]
enum AccountStatus {
    Initialized,
    Active,
    Deleted,
}

#[derive(Serialize)]
struct Account {
    account_id: Option<Uuid>,
    status: AccountStatus,
}

impl Account {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Account>, sqlx::Error> {
        let mut accounts = vec![];
        let recs = sqlx::query!(
            r#"
                SELECT account_id
                    FROM accounts
                ORDER BY account_id
            "#
        )
        .fetch_all(pool)
        .await?;

        for rec in recs {
            accounts.push(Account {
                account_id: Some(rec.account_id),
                status: AccountStatus::Initialized,
            });
        }

        Ok(accounts)
    }

    pub async fn create(account: &AccountOpened, pool: &PgPool) -> Result<Account, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let todo =
            sqlx::query("INSERT INTO accounts (account_id) VALUES ($1) RETURNING account_id")
                .bind(&account.account_id)
                .map(|row: PgRow| Account {
                    account_id: row.get(0),
                    status: AccountStatus::Active,
                })
                .fetch_one(&mut tx)
                .await?;

        tx.commit().await?;
        Ok(todo)
    }
}

// implementation of Actix Responder for Todo struct so we can return Todo from action handler
impl Responder for Account {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
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

impl CommandExecutor<DeleteAccount> for Account {
    fn execute(
        cmd: DeleteAccount,
        _state: &Self,
    ) -> Result<Vec<AccountDeleted>, CommandExecutorError> {
        Ok(vec![AccountDeleted {
            account_id: cmd.account_id,
        }])
    }
}

impl CommandExecutor<OpenAccount> for Account {
    fn execute(cmd: OpenAccount, state: &Self) -> Result<Vec<AccountOpened>, CommandExecutorError> {
        log::trace!("NEW ACCOUNT CREATED");
        match state.status {
            AccountStatus::Initialized => Ok(vec![AccountOpened {
                account_id: cmd.account_id,
                name: cmd.name,
            }]),
            _ => Err(CommandExecutorError::Any),
        }
    }
}

impl CommandExecutor<UpdateAccount> for Account {
    fn execute(
        _cmd: UpdateAccount,
        _state: &Self,
    ) -> Result<Vec<AccountUpdated>, CommandExecutorError> {
        Ok(vec![
            AccountUpdated::Deleted,
            AccountUpdated::Forced { why: "duno".into() },
        ])
    }
}

impl EventApplier<AccountOpened> for Account {
    fn apply(&mut self, event: &AccountOpened) -> Result<(), ApplyError> {
        self.account_id = Some(event.account_id);
        self.status = AccountStatus::Active;

        Ok(())
    }
}

impl EventApplier<AccountUpdated> for Account {
    fn apply(&mut self, _event: &AccountUpdated) -> Result<(), ApplyError> {
        Ok(())
    }
}
impl EventApplier<AccountDeleted> for Account {
    fn apply(&mut self, _event: &AccountDeleted) -> Result<(), ApplyError> {
        self.status = AccountStatus::Deleted;

        Ok(())
    }
}

#[derive(Debug)]
struct DeleteAccount {
    account_id: Uuid,
}

#[async_trait::async_trait]
impl Command for DeleteAccount {
    type Event = AccountDeleted;
    type Executor = Account;
    type ExecutorRegistry = AggregateInstanceRegistry<Account>;

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

#[derive(Clone, Debug, chekov::macros::Command, Serialize, Deserialize)]
#[command(event = "AccountUpdated", aggregate = "Account")]
struct UpdateAccount {
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

#[derive(Clone, Message, Debug, chekov::macros::Event, Deserialize, Serialize)]
#[rtype(result = "()")]
struct AccountOpened {
    account_id: Uuid,
    name: String,
}

#[derive(Clone, Message, Debug, chekov::macros::Event, Deserialize, Serialize)]
#[rtype(result = "()")]
enum AccountUpdated {
    Deleted,
    Forced { why: String },
    Disabled(String),
}

#[derive(chekov::macros::Event, Debug, Deserialize, Serialize)]
#[event(event_type = "Elixir.Conduit.Accounts.Events.UserRegistered")]
struct UserRegistered {
    email: String,
    hashed_password: String,
    user_uuid: Uuid,
    username: String,
}

#[get("/accounts")]
async fn find_all(db_pool: web::Data<PgPool>) -> impl Responder {
    let result = Account::find_all(db_pool.get_ref()).await;
    match result {
        Ok(todos) => HttpResponse::Ok().json(todos),
        _ => HttpResponse::BadRequest().body("Error trying to read all todos from database"),
    }
}
#[get("/accounts/{id}")]
async fn find(_id: web::Path<i32>) -> impl Responder {
    HttpResponse::InternalServerError().body("Unimplemented")
}

#[post("/accounts")]
async fn create(account: web::Json<OpenAccount>) -> impl Responder {
    // Router::<DefaultApp>::dispatch(account.clone()).await;
    // let cmd = UpdateAccount {
    //     account_id: account.account_id,
    //     name: "DELETED".into(),
    // };
    match Router::<DefaultApp>::dispatch(account.clone(), CommandMetadatas::default()).await {
        Ok(res) => HttpResponse::Ok().json(res), // <- send response
        Err(e) => HttpResponse::Ok().json(e),    // <- send response
    }
}

#[put("/accounts/{id}")]
async fn update(_id: web::Path<i32>, _account: web::Json<OpenAccount>) -> impl Responder {
    HttpResponse::InternalServerError().body("Unimplemented")
}

#[delete("/accounts/{id}")]
async fn delete(id: web::Path<uuid::Uuid>) -> impl Responder {
    match Router::<DefaultApp>::dispatch(
        DeleteAccount { account_id: id.0 },
        CommandMetadatas::default(),
    )
    .await
    {
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

struct AccountProjector {
    pool: PgPool,
}

impl EventHandler for AccountProjector {
    fn started<A: Application>(
        &mut self,
        ctx: &mut actix::Context<chekov::event_handler::EventHandlerInstance<A, Self>>,
    ) {
        self.listen::<_, AccountOpened>(ctx);
        self.listen::<_, AccountUpdated>(ctx);
    }
}

impl chekov::event_handler::Listening for AccountProjector {}

#[async_trait::async_trait]
impl chekov::event::Handler<AccountOpened> for AccountProjector {
    async fn handle(&self, event: &AccountOpened) -> Result<(), ()> {
        let _result = Account::create(event, &self.pool).await;
        Ok(())
    }
}

#[derive(Default)]
struct DefaultApp {}
impl chekov::Application for DefaultApp {
    type Storage = PostgresBackend;
}

#[derive(Default)]
struct UnstartedApp {}
impl chekov::Application for UnstartedApp {
    type Storage = PostgresBackend;
}
#[allow(dead_code)]
fn configure_events() -> HashMap<String, TypeId> {
    let mut hash = HashMap::new();

    hash.insert("AccountOpened".into(), TypeId::of::<AccountOpened>());
    hash.insert(
        "AccountUpdated::Forced".into(),
        TypeId::of::<AccountUpdated>(),
    );
    hash.insert(
        "AccountUpdated::Deleted".into(),
        TypeId::of::<AccountUpdated>(),
    );
    hash.insert(
        "AccountUpdated::Disabled".into(),
        TypeId::of::<AccountUpdated>(),
    );

    hash
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let db_pool = PgPool::connect("postgresql://postgres:postgres@localhost/bank")
        .await
        .unwrap();

    // Configure the storage (PG, InMemory,...)
    DefaultApp::with_default()
        .storage(PostgresBackend::with_url(
            "postgresql://postgres:postgres@localhost/event_store_bank",
        ))
        // .events(configure_events())
        // .event_handler(AccountProjector {
        //     pool: db_pool.clone(),
        // })
        .launch()
        .await;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            // .data(chekov.clone())
            .data(db_pool.clone())
            .data(web::JsonConfig::default().limit(4096))
            .configure(init::<PostgresBackend>)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
