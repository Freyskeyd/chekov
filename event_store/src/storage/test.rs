use uuid::Uuid;

use crate::storage::{
    inmemory::InMemoryBackend, Storage, StreamCreationError, StreamDeletionError,
};

mod creation {
    use super::*;

    #[test]
    fn success() {
        let mut storage = InMemoryBackend::default();
        let uuid = Uuid::new_v4().to_string();

        assert!(storage.create_stream(&uuid).is_ok());
    }

    #[test]
    fn fail_if_stream_exists() {
        let mut storage = InMemoryBackend::default();

        let uuid = Uuid::new_v4().to_string();

        assert!(storage.create_stream(&uuid).is_ok());
        assert_eq!(
            storage.create_stream(&uuid),
            Err(StreamCreationError::AlreadyExists)
        );
    }

    #[test]
    fn fail_if_stream_uuid_malformed() {
        let mut storage = InMemoryBackend::default();

        assert_eq!(
            storage.create_stream("an uuid"),
            Err(StreamCreationError::MalformedStreamUUID)
        );
    }
}

mod deletion {
    use super::*;

    #[test]
    fn success() {
        let mut storage = InMemoryBackend::default();
        let uuid = Uuid::new_v4().to_string();

        assert!(storage.create_stream(&uuid).is_ok());
        assert!(storage.delete_stream(&uuid).is_ok());
    }

    #[test]
    fn fail_if_stream_doesnt_exists() {
        let mut storage = InMemoryBackend::default();

        let uuid = Uuid::new_v4().to_string();

        assert_eq!(
            storage.delete_stream(&uuid),
            Err(StreamDeletionError::DoesntExists)
        );
    }

    #[test]
    fn fail_if_stream_uuid_malformed() {
        let mut storage = InMemoryBackend::default();

        assert_eq!(
            storage.delete_stream("an uuid"),
            Err(StreamDeletionError::MalformedStreamUUID)
        );
    }
}
