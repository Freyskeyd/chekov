use crate::stream::Stream;

#[derive(Debug)]
pub enum ReadVersion {
    Origin,
    Version(i64),
}

/// The `ExpectedVersion` used to define optimistic concurrency
#[derive(Debug, PartialEq)]
pub enum ExpectedVersion {
    /// Define that we expect a stream in any version
    AnyVersion,
    /// Define that we expect a non existing stream
    NoStream,
    /// Define that we expect an existing stream
    StreamExists,
    /// Define that we expect a stream in a particular version
    Version(i64),
}

impl ExpectedVersion {
    #[must_use]
    pub const fn verify(stream: &Stream, expected: &Self) -> ExpectedVersionResult {
        match expected {
            _ if stream.is_persisted() && stream.deleted_at.is_some() => {
                ExpectedVersionResult::StreamDeleted
            }
            Self::NoStream | Self::Version(0) | Self::AnyVersion if !stream.is_persisted() => {
                ExpectedVersionResult::NeedCreation
            }

            Self::StreamExists if !stream.is_persisted() => {
                ExpectedVersionResult::StreamDoesntExists
            }

            Self::AnyVersion | Self::StreamExists if stream.is_persisted() => {
                ExpectedVersionResult::Ok
            }

            Self::Version(version)
                if stream.is_persisted() && stream.stream_version == *version =>
            {
                ExpectedVersionResult::Ok
            }

            Self::NoStream if stream.is_persisted() && stream.stream_version != 0 => {
                ExpectedVersionResult::StreamAlreadyExists
            }

            Self::NoStream if stream.is_persisted() && stream.stream_version == 0 => {
                ExpectedVersionResult::Ok
            }
            _ => ExpectedVersionResult::WrongExpectedVersion,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ExpectedVersionResult {
    NeedCreation,
    Ok,
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

        assert_eq!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::AnyVersion),
            ExpectedVersionResult::NeedCreation
        );
    }

    #[test]
    fn test_validation_result_need_creation() {
        let mut stream = Stream::from_str("stream_1").unwrap();
        stream.stream_id = 1;
        stream.deleted_at = Some(Utc::now());

        assert_eq!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::AnyVersion),
            ExpectedVersionResult::StreamDeleted
        );

        stream.stream_id = 0;
        stream.deleted_at = None;

        assert_eq!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::NoStream),
            ExpectedVersionResult::NeedCreation
        );

        assert_eq!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::Version(0)),
            ExpectedVersionResult::NeedCreation
        );

        assert_eq!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::AnyVersion),
            ExpectedVersionResult::NeedCreation
        );

        assert_eq!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::StreamExists),
            ExpectedVersionResult::StreamDoesntExists
        );

        stream.stream_id = 1;
        assert_eq!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::AnyVersion),
            ExpectedVersionResult::Ok
        );

        assert_eq!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::StreamExists),
            ExpectedVersionResult::Ok
        );

        stream.stream_version = 1;

        assert_eq!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::Version(1)),
            ExpectedVersionResult::Ok
        );

        assert_eq!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::NoStream),
            ExpectedVersionResult::StreamAlreadyExists
        );

        stream.stream_version = 0;

        assert_eq!(
            ExpectedVersion::verify(&stream, &ExpectedVersion::NoStream),
            ExpectedVersionResult::Ok
        );
    }
}
