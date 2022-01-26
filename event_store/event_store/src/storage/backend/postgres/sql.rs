use crate::core::event::RecordedEvent;
use crate::core::event::UnsavedEvent;
use crate::core::stream::Stream;
use log::trace;
use sqlx::postgres::PgRow;
use sqlx::Row;
use std::convert::TryInto;
use uuid::Uuid;

pub async fn read_stream(
    conn: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    stream_id: i64,
    version: usize,
    limit: usize,
) -> Result<Vec<RecordedEvent>, sqlx::Error> {
    let version: i64 = version.try_into().unwrap();
    let limit: i64 = limit.try_into().unwrap();
    trace!("Version {}, Limit: {}", version, limit);
    #[allow(clippy::used_underscore_binding)]
    sqlx::query_as::<_, RecordedEvent>(
        r#"SELECT
        stream_events.stream_version as event_number,
        events.event_id as event_uuid,
        streams.stream_uuid,
        stream_events.original_stream_version as stream_version,
        events.event_type::text,
        events.correlation_id,
        events.causation_id,
        events.data::jsonb,
        events.metadata::text,
        events.created_at
    FROM
	stream_events
	inner JOIN streams ON streams.stream_id = stream_events.original_stream_id
	inner JOIN events ON stream_events.event_id = events.event_id
    WHERE
	stream_events.stream_id = $1 AND stream_events.stream_version >=$2
	ORDER BY stream_events.stream_version ASC
        LIMIT $3;
         "#,
    )
    .bind(&stream_id)
    .bind(&version)
    .bind(&limit)
    .fetch_all(conn)
    .await
}

pub async fn stream_info(
    conn: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    stream_uuid: &str,
) -> Result<Stream, sqlx::Error> {
    #[allow(clippy::used_underscore_binding)]
    #[allow(clippy::similar_names)]
    sqlx::query_as::<_, Stream>(
        "SELECT stream_id, stream_uuid, stream_version, created_at, deleted_at FROM streams WHERE stream_uuid = $1"
    ).bind(&stream_uuid)
        .fetch_one(conn)
        .await
}

pub async fn create_stream(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    stream_uuid: &str,
) -> Result<Stream, sqlx::Error> {
    #[allow(clippy::used_underscore_binding)]
    sqlx::query_as::<_, Stream>("INSERT INTO streams (stream_uuid) VALUES ($1) RETURNING stream_id, stream_uuid, stream_version, created_at, deleted_at")
      .bind(&stream_uuid)
      .fetch_one(pool)
      .await
}

fn create_append_indexes(events: usize) -> String {
    (0..events)
        .map(|i| {
            format!(
                "(${}, ${}, ${}, ${}, ${}::jsonb, ${}::jsonb, ${}::timestamp)",
                (i * 7) + 1,
                (i * 7) + 2,
                (i * 7) + 3,
                (i * 7) + 4,
                (i * 7) + 5,
                (i * 7) + 6,
                (i * 7) + 7
            )
        })
        .collect::<Vec<String>>()
        .join(",")
}

fn create_event_id_stream_version_indexes(starting_at: usize, length: usize) -> String {
    (0..length)
        .map(|i| match i {
            0 => format!("(${}::uuid, ${}::bigint)", starting_at, starting_at + 1),
            event_number => {
                let index = (event_number * 2) + starting_at;
                format!("(${}, ${})", index, index + 1)
            }
        })
        .collect::<Vec<String>>()
        .join(",")
}

pub async fn insert_events(
    conn: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    stream_uuid: &str,
    events: &[UnsavedEvent],
) -> Result<Vec<Uuid>, sqlx::Error> {
    let q = format!(
        r#"
WITH
  inserted_events AS (
    INSERT INTO events (event_id, event_type, causation_id, correlation_id, data, metadata, created_at)
    VALUES {} RETURNING *
  ),

  updated_stream AS (
    UPDATE
      streams
    SET
      stream_version = stream_version + {event_number}
    FROM inserted_events
    WHERE
      streams.stream_uuid = '{}' RETURNING streams.stream_id,
      stream_version,
      stream_version - {event_number} AS initial_stream_version,
      stream_uuid
  ),

  events_mapping (event_id, stream_version) AS (VALUES {events}),

  insert_stream_events AS (
    INSERT INTO stream_events (event_id, stream_id, stream_version, original_stream_id, original_stream_version)
    SELECT events_mapping.event_id, updated_stream.stream_id, events_mapping.stream_version, updated_stream.stream_id, events_mapping.stream_version
    FROM events_mapping, updated_stream RETURNING *
  )

SELECT
  inserted_events.event_id as event_uuid
FROM
  inserted_events;
  "#,
        create_append_indexes(events.len()),
        &stream_uuid,
        event_number = events.len(),
        events = create_event_id_stream_version_indexes(events.len() * 7 + 1, events.len()),
    );

    let mut query = sqlx::query(&q);
    for event in events {
        query = query
            .bind(event.event_uuid)
            .bind(&event.event_type)
            .bind(event.causation_id)
            .bind(event.correlation_id)
            .bind(&event.data)
            .bind(&event.metadata)
            .bind(&event.created_at);
    }
    for event in events.iter() {
        query = query.bind(event.event_uuid).bind(event.stream_version);
    }

    query.map(|row: PgRow| row.get(0)).fetch_all(conn).await
}

#[cfg(test)]
mod test {

    use super::create_append_indexes;
    use crate::prelude::*;
    use serde::{Deserialize, Serialize};

    use pretty_assertions::assert_eq;

    #[derive(Serialize, Deserialize)]
    struct MyEvent {}
    impl Event for MyEvent {
        fn event_type(&self) -> &'static str {
            "MyEvent"
        }

        fn all_event_types() -> Vec<&'static str> {
            vec!["MyEvent"]
        }
    }

    impl std::convert::TryFrom<crate::prelude::RecordedEvent> for MyEvent {
        type Error = ();
        fn try_from(e: crate::prelude::RecordedEvent) -> Result<Self, Self::Error> {
            serde_json::from_value(e.data).map_err(|_| ())
        }
    }

    #[test]
    fn should_produce_the_right_number_of_arguments() {
        let e = MyEvent {};
        let events: Vec<UnsavedEvent> = vec![
            UnsavedEvent::try_from(&e).unwrap(),
            UnsavedEvent::try_from(&e).unwrap(),
        ];
        assert_eq!(
            "($1, $2, $3, $4, $5::jsonb, $6::jsonb, $7::timestamp),($8, $9, $10, $11, $12::jsonb, $13::jsonb, $14::timestamp)",
            create_append_indexes(events.len())
        );
    }
}
