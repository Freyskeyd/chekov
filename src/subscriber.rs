use actix::prelude::*;
use sqlx::postgres::PgNotification;
use uuid::Uuid;

mod listener;
mod manager;
mod subscriber;

pub use listener::Listener;
pub use manager::SubscriberManager;
pub use subscriber::Subscriber;

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct Notif(PgNotification);

#[doc(hidden)]
#[derive(Message)]
#[rtype(result = "()")]
struct Subscribe {
    subscriber_id: Uuid,
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct EventNotification {
    stream_id: i32,
    stream_uuid: String,
    first_stream_version: i32,
    last_stream_version: i32,
}

trait IsEmptyResult {
    fn is_empty_result<E>(self, err: E) -> Result<Self, E>
    where
        Self: std::marker::Sized;
}

impl<'a> IsEmptyResult for &'a str {
    fn is_empty_result<E>(self, err: E) -> Result<Self, E> {
        if self.is_empty() {
            Err(err)
        } else {
            Ok(self)
        }
    }
}

use std::convert::TryFrom;

impl<'a> TryFrom<&'a str> for EventNotification {
    type Error = &'static str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let elements = value.splitn(4, ',').collect::<Vec<&str>>();

        let mut through = elements.iter();

        let stream_uuid = through
            .next()
            .ok_or("unable to parse stream_uuid")?
            .is_empty_result("unable to parse stream_uuid")?
            .to_owned();

        let stream_id = through
            .next()
            .ok_or("unable to parse stream_id")?
            .parse::<i32>()
            .or(Err("unable to convert stream_id to i32"))?;

        let first_stream_version = through
            .next()
            .ok_or("unable to parse first_stream_version")?
            .parse::<i32>()
            .or(Err("unable to convert first_stream_version to i32"))?;

        let last_stream_version = through
            .next()
            .ok_or("unable to parse last_stream_version")?
            .parse::<i32>()
            .or(Err("unable to convert last_stream_version to i32"))?;

        Ok(Self {
            stream_uuid,
            stream_id,
            first_stream_version,
            last_stream_version,
        })
    }
}
