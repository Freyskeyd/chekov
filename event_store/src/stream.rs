use chrono::DateTime;

/// A `Stream` represents an `Event` stream
#[derive(Clone, Debug, PartialEq)]
pub struct Stream {
    pub(crate) stream_id: i64,
    /// The stream identifier which is unique
    pub(crate) stream_uuid: String,
    /// The current stream version number
    pub(crate) stream_version: i64,
    /// The creation date of the stream
    pub(crate) created_at: Option<DateTime<chrono::offset::Utc>>,
    /// The deletion date of the stream
    pub(crate) deleted_at: Option<DateTime<chrono::offset::Utc>>,
}

impl Stream {
    #[must_use]
    pub fn stream_uuid(&self) -> &str {
        self.stream_uuid.as_ref()
    }

    pub(crate) const fn is_persisted(&self) -> bool {
        self.stream_id != 0
    }

    pub(crate) fn validates_stream_id(stream_id: &str) -> bool {
        !stream_id.contains(' ')
    }
}

#[derive(Debug, PartialEq)]
pub enum StreamError {
    MalformedStreamUUID,
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
            created_at: None,
            deleted_at: None,
        })
    }
}
