use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use chekov::prelude::*;
use chekov::EventHandler;
use event_store::prelude::*;
use futures::future::{ready, Ready};
use futures::TryFutureExt;
use serde::Deserialize;
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;
// mod user;

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
        match state.status {
            AccountStatus::Initialized => Ok(vec![AccountOpened {
                account_id: cmd.account_id,
                name: cmd.name,
            }]),
            _ => Err(CommandExecutorError::Any),
        }
    }
}

impl EventApplier<AccountOpened> for Account {
    fn apply(&mut self, event: &AccountOpened) -> Result<(), ApplyError> {
        self.account_id = Some(event.account_id);
        self.status = AccountStatus::Active;

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

type PGChekov = Chekov<PostgresBackend>;

use actix_web::{delete, get, post, put};
use actix_web::{middleware, web, App, HttpServer};

#[get("/accounts")]
async fn find_all(db_pool: web::Data<PgPool>) -> impl Responder {
    let result = Account::find_all(db_pool.get_ref()).await;
    match result {
        Ok(todos) => HttpResponse::Ok().json(todos),
        _ => HttpResponse::BadRequest().body("Error trying to read all todos from database"),
    }
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
    account: web::Json<OpenAccount>,
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

struct AccountProjector {
    pool: PgPool,
}

impl chekov::EventHandler for AccountProjector {}

#[async_trait::async_trait]
impl chekov::event::Handler<AccountOpened> for AccountProjector {
    async fn handle(&self, event: &AccountOpened) -> Result<(), ()> {
        let _result = Account::create(event, &self.pool).await;
        Ok(())
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    // Configure the storage (PG, InMemory,...)
    let storage = PostgresBackend::with_url("postgresql://postgres:postgres@localhost/event_store")
        .await
        .unwrap();

    let app = Chekov::with_storage(storage).await;

    let db_pool = PgPool::connect("postgresql://postgres:postgres@localhost/bank")
        .await
        .unwrap();

    AccountProjector {
        pool: db_pool.clone(),
    }
    .builder()
    .name("AccountProjector")
    .register(&app)
    .await;
    let chekov = std::sync::Arc::new(app);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(chekov.clone())
            .data(db_pool.clone())
            .data(web::JsonConfig::default().limit(4096))
            .configure(init::<PostgresBackend>)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
