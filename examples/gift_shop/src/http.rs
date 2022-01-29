use super::*;
use crate::account::*;
use crate::commands::*;
use actix_web::body::BoxBody;
use actix_web::post;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse, Responder};
use uuid::Uuid;

impl Responder for Account {
    type Body = BoxBody;
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
    }
}

#[post("/accounts")]
pub async fn create_account(account: web::Json<OpenAccountPayload>) -> impl Responder {
    match Router::<DefaultApp>::dispatch::<OpenAccount>(
        account.into_inner().into(),
        CommandMetadatas::default(),
    )
    .await
    {
        Ok(res) => HttpResponse::Ok().json(res.first()),
        Err(e) => HttpResponse::Ok().json(e),
    }
}

#[post("/gift_cards")]
pub async fn create_gift_card(gift_card: web::Json<CreateGiftCardPayload>) -> impl Responder {
    match Router::<DefaultApp>::dispatch::<CreateGiftCard>(
        gift_card.into_inner().into(),
        CommandMetadatas::default(),
    )
    .await
    {
        Ok(res) => HttpResponse::Ok().json(res.first()),
        Err(e) => HttpResponse::Ok().json(e),
    }
}

#[post("/accounts/{account_id}/orders")]
pub async fn create_order(params: web::Path<Uuid>) -> impl Responder {
    match Router::<DefaultApp>::dispatch::<CreateOrder>(
        CreateOrder {
            order_id: Uuid::new_v4(),
            account_id: params.into_inner(),
        },
        CommandMetadatas::default(),
    )
    .await
    {
        Ok(res) => HttpResponse::Ok().json(res.first()),
        Err(e) => HttpResponse::Ok().json(e),
    }
}

#[post("/orders/{order_id}/items")]
pub async fn add_order_item(
    params: web::Path<Uuid>,
    payload: web::Json<AddGiftCardToOrderPayload>,
) -> impl Responder {
    let payload = payload.into_inner();
    match Router::<DefaultApp>::dispatch::<AddGiftCardToOrder>(
        AddGiftCardToOrder {
            order_id: params.into_inner(),
            gift_card_id: payload.gift_card_id,
            amount: payload.amount,
            price: 10,
        },
        CommandMetadatas::default(),
    )
    .await
    {
        Ok(res) => HttpResponse::Ok().json(res.first()),
        Err(e) => HttpResponse::Ok().json(e),
    }
}

#[derive(serde::Deserialize)]
pub struct OpenAccountPayload {
    name: String,
}

#[derive(serde::Deserialize)]
pub struct CreateGiftCardPayload {
    name: String,
    price: i64,
    count: usize,
}

#[derive(serde::Deserialize)]
pub struct AddGiftCardToOrderPayload {
    gift_card_id: Uuid,
    amount: usize,
}

impl From<OpenAccountPayload> for OpenAccount {
    fn from(payload: OpenAccountPayload) -> Self {
        Self {
            account_id: Uuid::new_v4(),
            name: payload.name,
        }
    }
}

impl From<CreateGiftCardPayload> for CreateGiftCard {
    fn from(payload: CreateGiftCardPayload) -> Self {
        Self {
            gift_card_id: Uuid::new_v4(),
            name: payload.name,
            price: payload.price,
            count: payload.count,
        }
    }
}
