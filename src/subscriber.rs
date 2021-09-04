use actix::prelude::*;
use sqlx::postgres::PgNotification;
use std::{convert::TryFrom, str::FromStr};

mod listener;
mod manager;
mod subscriber;

pub use listener::Listener;
pub use manager::SubscriberManager;
pub use subscriber::Subscriber;

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct Notif(PgNotification);

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct EventNotification {
    stream_id: i32,
    stream_uuid: String,
    first_stream_version: i32,
    last_stream_version: i32,
}

struct NonEmptyString(String);

impl FromStr for NonEmptyString {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("unable to parse stream_uuid")
        } else {
            Ok(NonEmptyString(s.to_owned()))
        }
    }
}
impl From<NonEmptyString> for String {
    fn from(s: NonEmptyString) -> Self {
        s.0
    }
}

impl<'a> TryFrom<&'a str> for EventNotification {
    type Error = &'static str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut through = value.splitn(4, ',');

        let stream_uuid = through
            .next()
            .ok_or("unable to parse stream_uuid")?
            .parse::<NonEmptyString>()?
            .into();

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
