use chrono::DateTime;

pub mod error;

use error::StreamError;

/// A `Stream` represents an `Event` stream
#[derive(Clone, Debug, PartialEq, sqlx::FromRow)]
pub struct Stream {
    #[sqlx(try_from = "i64")]
    pub stream_id: u64,
    /// The stream identifier which is unique
    pub stream_uuid: String,
    /// The current stream version number
    #[sqlx(try_from = "i64")]
    pub stream_version: u64,
    /// The creation date of the stream
    pub created_at: DateTime<chrono::offset::Utc>,
    /// The deletion date of the stream
    pub deleted_at: Option<DateTime<chrono::offset::Utc>>,
}

impl Stream {
    #[must_use]
    pub fn stream_uuid(&self) -> &str {
        self.stream_uuid.as_ref()
    }

    pub const fn is_persisted(&self) -> bool {
        self.stream_id != 0
    }

    pub fn validates_stream_id(stream_id: &str) -> bool {
        !stream_id.contains(' ')
    }
}

impl std::str::FromStr for Stream {
    type Err = StreamError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !Self::validates_stream_id(s) {
            return Err(StreamError::MalformedStreamUUID);
        }

        Ok(Self {
            stream_id: 0,
            stream_uuid: s.into(),
            stream_version: 0,
            created_at: chrono::Utc::now(),
            deleted_at: None,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::str::FromStr;

    #[test]
    fn test_stream_from_str() {
        assert_eq!(
            Err(StreamError::MalformedStreamUUID),
            Stream::from_str("unvalid stream name")
        );
    }

    #[test]
    fn test_stream_is_not_persisted_by_default() {
        assert!(!Stream::from_str("unvalid").unwrap().is_persisted());
    }
}
