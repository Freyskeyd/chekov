use crate::event::UnsavedEvent;
use crate::stream::Stream;
use sqlx::postgres::PgRow;
use sqlx::Row;
use uuid::Uuid;

pub async fn stream_info<'c, C>(conn: C, stream_uuid: &str) -> Result<Stream, sqlx::Error>
where
    C: sqlx::Executor<Database = sqlx::Postgres>
        + sqlx::executor::RefExecutor<'c, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        Stream,
        "SELECT stream_id, stream_uuid, stream_version, created_at, deleted_at FROM streams WHERE stream_uuid = $1",
        stream_uuid
    )
        .fetch_one(conn)
        .await
}

pub async fn create_stream(pool: &sqlx::PgPool, stream_uuid: &str) -> Result<Stream, sqlx::Error> {
    sqlx::query_as!(Stream, "INSERT INTO streams (stream_uuid) VALUES ($1) RETURNING stream_id, stream_uuid, stream_version, created_at, deleted_at", stream_uuid).fetch_one(pool).await
}

pub async fn insert_stream_events<'c, C>(
    tx: C,
    events: &[UnsavedEvent],
    stream_id: i64,
) -> Result<u64, sqlx::Error>
where
    C: sqlx::Executor<Database = sqlx::Postgres>
        + sqlx::executor::RefExecutor<'c, Database = sqlx::Postgres>,
{
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

    query.execute(tx).await
}

pub async fn insert_link_events<'t, C>(
    _tx: C,
    _events: &[Uuid],
    _stream_uuid: &str,
) -> Result<(), sqlx::Error>
where
    C: sqlx::Executor<Database = sqlx::Postgres>,
{
    Ok(())
}

pub async fn transactional_insert_events(
    tx: &mut sqlx::Transaction<sqlx::pool::PoolConnection<sqlx::postgres::PgConnection>>,
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

    query.map(|row: PgRow| row.get(0)).fetch_all(tx).await
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
    use serde::Serialize;

    #[derive(Serialize)]
    struct MyEvent {}
    impl Event for MyEvent {
        fn event_type(&self) -> &'static str {
            "MyEvent"
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
