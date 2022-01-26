use std::{pin::Pin, str::FromStr};

use actix::{Addr, Message};
use futures::{future::BoxFuture, Future, Stream};

use crate::{event::RecordedEvent, storage::Storage};

pub trait EventBus: std::fmt::Debug + Default + Send + std::marker::Unpin + 'static {
    fn bus_name() -> &'static str;

    // fn prepare<F: Future<Output = ()> + 'static>(&mut self, resolver: F) -> BoxFuture<'static, ()>;

    fn create_stream(&mut self) -> BoxedStream;
}

pub type BoxedStream =
    Pin<Box<dyn Future<Output = Pin<Box<dyn Stream<Item = Result<EventBusMessage, ()>>>>>>>;

#[derive(Debug, Message)]
#[rtype("()")]
pub enum EventBusMessage {
    Notification(EventNotification),
    Events(Vec<RecordedEvent>),
    Unkown,
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct EventNotification {
    pub stream_id: i32,
    pub stream_uuid: String,
    pub first_stream_version: i32,
    pub last_stream_version: i32,
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
