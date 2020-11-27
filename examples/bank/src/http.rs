use super::*;
use crate::account::*;
use crate::commands::*;
use actix_web::web;
use actix_web::{delete, get, post, put};
use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use sqlx::PgPool;

impl Responder for Account {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

#[get("/accounts")]
pub async fn find_all(db_pool: web::Data<PgPool>) -> impl Responder {
    let result = Account::find_all(db_pool.get_ref()).await;
    match result {
        Ok(todos) => HttpResponse::Ok().json(todos),
        _ => HttpResponse::BadRequest().body("Error trying to read all todos from database"),
    }
}
#[get("/accounts/{id}")]
pub async fn find(_id: web::Path<i32>) -> impl Responder {
    HttpResponse::InternalServerError().body("Unimplemented")
}

#[post("/accounts")]
pub async fn create(account: web::Json<OpenAccount>) -> impl Responder {
    match Router::<DefaultApp>::dispatch(account.clone(), CommandMetadatas::default()).await {
        Ok(res) => HttpResponse::Ok().json(res), // <- send response
        Err(e) => HttpResponse::Ok().json(e),    // <- send response
    }
}

#[put("/accounts/{id}")]
pub async fn update(_id: web::Path<i32>, _account: web::Json<OpenAccount>) -> impl Responder {
    HttpResponse::InternalServerError().body("Unimplemented")
}

#[delete("/accounts/{id}")]
pub async fn delete(id: web::Path<uuid::Uuid>) -> impl Responder {
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
