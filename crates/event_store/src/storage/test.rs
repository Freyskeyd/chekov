use uuid::Uuid;

use crate::core::stream::Stream;

use std::str::FromStr;

mod creation {
    use super::*;

    use event_store_backend_inmemory::InMemoryBackend;
    use event_store_core::{backend::Backend, storage::StorageError};

    #[tokio::test]
    async fn success() {
        let mut storage = InMemoryBackend::default();
        let uuid = Uuid::new_v4().to_string();
        let c_id = Uuid::new_v4();

        assert!(storage
            .create_stream(Stream::from_str(&uuid).unwrap(), c_id)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn fail_if_stream_exists() {
        let mut storage = InMemoryBackend::default();

        let uuid = Uuid::new_v4().to_string();
        let c_id = Uuid::new_v4();

        assert!(storage
            .create_stream(Stream::from_str(&uuid).unwrap(), c_id)
            .await
            .is_ok());
        assert!(matches!(
            storage
                .create_stream(Stream::from_str(&uuid).unwrap(), c_id)
                .await,
            Err(StorageError::StreamAlreadyExists)
        ));
    }
}

mod deletion {
    use super::*;

    use event_store_backend_inmemory::InMemoryBackend;
    use event_store_core::{backend::Backend, storage::StorageError};

    #[tokio::test]
    async fn success() {
        let mut storage = InMemoryBackend::default();
        let uuid = Uuid::new_v4().to_string();
        let c_id = Uuid::new_v4();

        assert!(storage
            .create_stream(Stream::from_str(&uuid).unwrap(), c_id)
            .await
            .is_ok());
        assert!(storage
            .delete_stream(&Stream::from_str(&uuid).unwrap(), c_id)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn fail_if_stream_doesnt_exists() {
        let mut storage = InMemoryBackend::default();

        let uuid = Uuid::new_v4().to_string();
        let c_id = Uuid::new_v4();

        assert!(matches!(
            storage
                .delete_stream(&Stream::from_str(&uuid).unwrap(), c_id)
                .await,
            Err(StorageError::StreamDoesntExists)
        ));
    }
}
