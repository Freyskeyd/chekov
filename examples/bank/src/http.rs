use super::*;
use crate::account::*;
use crate::commands::*;
use actix_web::web;
use actix_web::{delete, get, post, put};
use actix_web::{HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

impl Responder for Account {
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
    }
}

#[get("/accounts")]
pub async fn find_all(db_pool: web::Data<PgPool>) -> impl Responder {
    let result = AccountRepository::find_all(db_pool.acquire().await.unwrap()).await;
    match result {
        Ok(todos) => HttpResponse::Ok().json(todos),
        _ => HttpResponse::BadRequest().body("Error trying to read all todos from database"),
    }
}
#[get("/accounts/{id}")]
pub async fn find(_id: web::Path<Uuid>) -> impl Responder {
    HttpResponse::InternalServerError().body("Unimplemented")
}

#[post("/accounts")]
pub async fn create(account: web::Json<OpenAccount>) -> impl Responder {
    match Router::<DefaultApp>::dispatch(account.into_inner(), CommandMetadatas::default()).await {
        Ok(res) => HttpResponse::Ok().json(res.first()), // <- send response
        Err(e) => HttpResponse::Ok().json(e),            // <- send response
    }
}

#[put("/accounts/{id}")]
pub async fn update(
    account_id: web::Path<Uuid>,
    account: web::Json<OpenAccount>,
) -> impl Responder {
    match Router::<DefaultApp>::dispatch(
        UpdateAccount {
            account_id: account_id.into_inner(),
            name: account.name.to_string(),
        },
        CommandMetadatas::default(),
    )
    .await
    {
        Ok(res) => HttpResponse::Ok().json(res.first()), // <- send response
        Err(e) => HttpResponse::Ok().json(e),            // <- send response
    }
}

#[delete("/accounts/{id}")]
pub async fn delete(id: web::Path<uuid::Uuid>) -> impl Responder {
    match Router::<DefaultApp>::dispatch(
        DeleteAccount {
            account_id: id.into_inner(),
        },
        CommandMetadatas::default(),
    )
    .await
    {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::Ok().json(e),
    }
}
