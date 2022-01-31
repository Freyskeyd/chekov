use std::pin::Pin;

use actix::Message;
use futures::{Future, Stream};

use crate::event::RecordedEvent;

use self::error::EventNotificationError;

pub mod error;

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

/// Notification produced by the `EventBus` which contains events/streams related informations
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct EventNotification {
    pub stream_id: i32,
    pub stream_uuid: String,
    pub first_stream_version: i32,
    pub last_stream_version: i32,
}

impl<'a> TryFrom<&'a str> for EventNotification {
    type Error = EventNotificationError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut through = value.splitn(4, ',');

        let stream_uuid = if let Ok(stream_uuid) = through
            .next()
            .ok_or(EventNotificationError::ParsingError {
                field: "stream_uuid",
            })?
            .parse::<String>()
        {
            if stream_uuid.is_empty() {
                return Err(EventNotificationError::InvalidStreamUUID);
            }
            stream_uuid
        } else {
            return Err(EventNotificationError::ParsingError {
                field: "stream_uuid",
            });
        };

        let stream_id = through
            .next()
            .ok_or(EventNotificationError::ParsingError { field: "stream_id" })?
            .parse::<i32>()
            .or(Err(EventNotificationError::ParsingError {
                field: "stream_id",
            }))?;

        let first_stream_version = through
            .next()
            .ok_or(EventNotificationError::ParsingError {
                field: "first_stream_version",
            })?
            .parse::<i32>()
            .or(Err(EventNotificationError::ParsingError {
                field: "first_stream_version",
            }))?;

        let last_stream_version = through
            .next()
            .ok_or(EventNotificationError::ParsingError {
                field: "last_stream_version",
            })?
            .parse::<i32>()
            .or(Err(EventNotificationError::ParsingError {
                field: "last_stream_version",
            }))?;

        Ok(Self {
            stream_uuid,
            stream_id,
            first_stream_version,
            last_stream_version,
        })
    }
}
