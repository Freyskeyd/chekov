use crate::event::RecordedEvent;
use crate::event::UnsavedEvent;
use crate::stream::Stream;
use sqlx::postgres::PgRow;
use sqlx::Done;
use sqlx::Row;
use std::convert::TryInto;
use uuid::Uuid;

pub async fn read_stream(
    conn: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    stream_uuid: &str,
    version: usize,
    limit: usize,
) -> Result<Vec<RecordedEvent>, sqlx::Error> {
    let version: i64 = version.try_into().unwrap();
    let limit: i64 = limit.try_into().unwrap();
    #[allow(clippy::used_underscore_binding)]
    sqlx::query_as!(
        RecordedEvent,
        r#"SELECT
        stream_events.stream_version as event_number,
        events.event_id as event_uuid,
        streams.stream_uuid,
        stream_events.original_stream_version as stream_version,
        events.event_type::text,
        events.correlation_id,
        events.causation_id,
        events.data::jsonb,
        events.metadata as "metadata: String",
        events.created_at
    FROM
	events
	inner JOIN stream_events ON stream_events.event_id = events.event_id
	inner JOIN streams ON streams.stream_id = stream_events.original_stream_id
    WHERE
	streams.stream_uuid = $1 AND stream_events.stream_version >=$2
	ORDER BY stream_events.stream_version ASC
        LIMIT $3;
         "#,
        &stream_uuid,
        &version,
        &limit,
    )
    .fetch_all(conn)
    .await
}
pub async fn stream_info(
    conn: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    stream_uuid: &str,
) -> Result<Stream, sqlx::Error> {
    #[allow(clippy::used_underscore_binding)]
    #[allow(clippy::similar_names)]
    sqlx::query_as!(
        Stream,
        "SELECT stream_id, stream_uuid, stream_version, created_at, deleted_at FROM streams WHERE stream_uuid = $1",
        &stream_uuid
    )
        .fetch_one(conn)
        .await
}

pub async fn create_stream(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // pool: &sqlx::PgPool,
    stream_uuid: &str,
) -> Result<Stream, sqlx::Error> {
    #[allow(clippy::used_underscore_binding)]
    sqlx::query_as!(Stream, "INSERT INTO streams (stream_uuid) VALUES ($1) RETURNING stream_id, stream_uuid, stream_version, created_at, deleted_at", &stream_uuid).fetch_one(pool).await
}

pub async fn insert_stream_events<'c>(
    tx: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    events: &[UnsavedEvent],
    stream_id: i64,
) -> Result<u64, sqlx::Error> {
    let params = (0..events.len())
        .map(|i| match i {
            0 => "($3::uuid, $4::bigint)".to_string(),
            event_number => {
                let index = event_number * 2 + 2;
                format!("(${}, ${})", index + 1, index + 2)
            }
        })
        .collect::<Vec<String>>()
        .join(",");

    let q = format!(
        "
        WITH
            stream AS (
                UPDATE streams
                SET stream_version = stream_version + $2::bigint
                WHERE stream_id = $1::bigint
                RETURNING stream_id
            ),
            events (event_id, stream_version) AS (VALUES {})
        INSERT INTO stream_events
                (
                event_id,
                stream_id,
                stream_version,
                original_stream_id,
                original_stream_version
                )
            SELECT
                events.event_id,
                stream.stream_id,
                events.stream_version,
                stream.stream_id,
                events.stream_version
            FROM events, stream;
            ",
        params
    );

    let mut query = sqlx::query(&q).bind(stream_id).bind(events.len() as i64);

    for event in events {
        query = query.bind(event.event_uuid).bind(&event.stream_version);
    }

    let done = query.execute(tx).await?;

    Ok(done.rows_affected())
}

pub async fn insert_link_events<'t>(
    tx: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    events: &[Uuid],
    stream_id: i64,
) -> Result<u64, sqlx::Error> {
    let params = (0..events.len())
        .map(|i| match i {
            0 => "($3::bigint, $4::uuid)".to_string(),
            event_number => {
                let index = event_number * 2 + 2;
                format!("(${}, ${})", index + 1, index + 2)
            }
        })
        .collect::<Vec<String>>()
        .join(",");

    let q = format!(
        "
WITH
  stream AS (
    UPDATE streams
    SET stream_version = stream_version + $2::bigint
    WHERE stream_id = $1::bigint
    RETURNING stream_version - $2 as initial_stream_version
  ),
  linked_events (index, event_id) AS (
    VALUES
    {}
  )
INSERT INTO stream_events
  (
    stream_id,
    stream_version,
    event_id,
    original_stream_id,
    original_stream_version
  )
SELECT
  $1,
  stream.initial_stream_version + linked_events.index,
  linked_events.event_id,
  original_stream_events.original_stream_id,
  original_stream_events.stream_version
FROM linked_events
CROSS JOIN stream
INNER JOIN stream_events as original_stream_events
  ON original_stream_events.event_id = linked_events.event_id
    AND original_stream_events.stream_id = original_stream_events.original_stream_id;
    ",
        params
    );

    let mut query = sqlx::query(&q).bind(stream_id).bind(events.len() as i64);

    for (index, event) in events.iter().enumerate() {
        query = query.bind(index as i64).bind(event);
    }

    let done = query.execute(tx).await?;

    Ok(done.rows_affected())
}

pub async fn transactional_insert_events(
    tx: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    events: &[UnsavedEvent],
) -> Result<Vec<Uuid>, sqlx::Error> {
    let q = format!("
                    INSERT INTO events
                    ( event_id, event_type, causation_id, correlation_id, data, metadata, created_at )
                    VALUES
                    {}

                    RETURNING event_id
                    ", create_append_indexes(events.len()));

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

    query
        .try_map(|row: PgRow| row.try_get(0))
        .fetch_all(tx)
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
