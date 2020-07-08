/// A `Stream` represents an `Event` stream
#[derive(Clone, Debug, PartialEq)]
pub struct Stream {
    /// The stream identifier which is unique
    pub(crate) stream_uuid: String,
    /// The current stream version number
    pub(crate) stream_version: i32,
    /// The creation date of the stream
    created_at: String,
    /// The deletion date of the stream
    deleted_at: String,
}

impl Stream {
    #[must_use]
    pub fn stream_uuid(&self) -> &str {
        self.stream_uuid.as_ref()
    }
}


#[derive(Debug, PartialEq)]
pub enum StreamError {
    MalformedStreamUUID,
}

impl std::str::FromStr for Stream {
    type Err = StreamError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(' ') {
            return Err(StreamError::MalformedStreamUUID);
        }

        Ok(Self {
            stream_uuid: s.into(),
            stream_version: 0,
            created_at: String::new(),
            deleted_at: String::new(),
        })
    }
}
