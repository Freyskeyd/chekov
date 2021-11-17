use uuid::Uuid;

use crate::storage::backend::Backend;
use crate::storage::{backend::inmemory::InMemoryBackend, StorageError};
use crate::stream::Stream;

use std::str::FromStr;

mod creation {
    use super::*;

    use pretty_assertions::assert_eq;

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
        assert_eq!(
            storage
                .create_stream(Stream::from_str(&uuid).unwrap(), c_id)
                .await,
            Err(StorageError::StreamAlreadyExists)
        );
    }
}

mod deletion {
    use super::*;

    use pretty_assertions::assert_eq;

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

        assert_eq!(
            storage
                .delete_stream(&Stream::from_str(&uuid).unwrap(), c_id)
                .await,
            Err(StorageError::StreamDoesntExists)
        );
    }
}
