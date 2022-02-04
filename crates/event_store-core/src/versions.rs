//! Contains every type to deal with versions in event_store.

use crate::stream::Stream;

/// Version used to read a stream.
///
/// A `ReadVersion` is a simple way to define at what cursor you want to read the stream.
///
/// `ReadVersion::Origin` is the same as `ReadVersion::Version(0)`
#[derive(Debug)]
pub enum ReadVersion {
    Origin,
    Version(i64),
}

/// The `ExpectedVersion` used to define optimistic concurrency
#[derive(Debug, PartialEq)]
pub enum ExpectedVersion {
    /// Define that we expect a stream in any version
    ///
    /// This expected version is always valid
    AnyVersion,
    /// Define that we expect a non existing stream
    ///
    /// This expected version is valid when a stream is persisted and the current version is 0
    NoStream,
    /// Define that we expect an existing stream
    ///
    /// This expected version is valid when the stream is persisted
    StreamExists,
    /// Define that we expect a stream in a particular version
    ///
    /// This expected version is valid when the stream is persisted and at the same version
    Version(i64),
}

impl ExpectedVersion {
    /// Verify if the given `Stream` match the `ExpectedVersion`
    #[must_use]
    pub const fn verify(stream: &Stream, expected: &Self) -> Result<(), ExpectedVersionError> {
        match expected {
            _ if stream.is_persisted() && stream.deleted_at.is_some() => {
                Err(ExpectedVersionError::StreamDeleted)
            }
            Self::NoStream | Self::Version(0) | Self::AnyVersion if !stream.is_persisted() => {
                Err(ExpectedVersionError::NeedCreation)
            }

            Self::StreamExists if !stream.is_persisted() => {
                Err(ExpectedVersionError::StreamDoesntExists)
            }

            Self::AnyVersion | Self::StreamExists if stream.is_persisted() => Ok(()),

            Self::Version(version)
                if stream.is_persisted() && stream.stream_version == *version =>
            {
                Ok(())
            }

            Self::NoStream if stream.is_persisted() && stream.stream_version != 0 => {
                Err(ExpectedVersionError::StreamAlreadyExists)
            }

            Self::NoStream if stream.is_persisted() && stream.stream_version == 0 => Ok(()),
            _ => Err(ExpectedVersionError::WrongExpectedVersion),
        }
    }
}

/// Error returned when the version verification fails
///
/// Those errors can define why the verification fails, it can be used to define what the
/// event_store need to do to be ready to proceed the query.
#[derive(Debug, PartialEq)]
pub enum ExpectedVersionError {
    NeedCreation,
    StreamAlreadyExists,
    StreamDeleted,
    StreamDoesntExists,
    WrongExpectedVersion,
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Utc;
    use std::str::FromStr;

    #[test]
    fn test_that_an_unexisting_stream_with_any_is_created() {
        let stream = Stream::from_str("stream_1").unwrap();

        assert!(matches!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::AnyVersion),
            Err(ExpectedVersionError::NeedCreation)
        ));
    }

    #[test]
    fn test_validation_result_need_creation() {
        let mut stream = Stream::from_str("stream_1").unwrap();
        stream.stream_id = 1;
        stream.deleted_at = Some(Utc::now());

        assert!(matches!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::AnyVersion),
            Err(ExpectedVersionError::StreamDeleted)
        ));

        stream.stream_id = 0;
        stream.deleted_at = None;

        assert!(matches!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::NoStream),
            Err(ExpectedVersionError::NeedCreation)
        ));

        assert!(matches!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::Version(0)),
            Err(ExpectedVersionError::NeedCreation)
        ));

        assert!(matches!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::AnyVersion),
            Err(ExpectedVersionError::NeedCreation)
        ));

        assert!(matches!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::StreamExists),
            Err(ExpectedVersionError::StreamDoesntExists)
        ));

        stream.stream_id = 1;
        assert!(ExpectedVersion::verify(&stream, &ExpectedVersion::AnyVersion).is_ok());
        assert!(ExpectedVersion::verify(&stream, &ExpectedVersion::StreamExists).is_ok());

        stream.stream_version = 1;

        assert!(ExpectedVersion::verify(&stream, &ExpectedVersion::Version(1)).is_ok());

        assert!(matches!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::NoStream),
            Err(ExpectedVersionError::StreamAlreadyExists)
        ));

        stream.stream_version = 0;

        assert!(ExpectedVersion::verify(&stream, &ExpectedVersion::NoStream).is_ok());
    }
}
