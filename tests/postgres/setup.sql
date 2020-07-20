CREATE TABLE streams (
	stream_id bigserial PRIMARY KEY NOT NULL,
	stream_uuid text NOT NULL,
	stream_version bigint DEFAULT 0 NOT NULL,
	created_at timestamp WITH time zone DEFAULT NOW(
) NOT NULL,
	deleted_at timestamp WITH time zone
);

CREATE UNIQUE INDEX ix_streams_stream_uuid ON streams (stream_uuid);

CREATE TABLE events (
	event_id uuid PRIMARY KEY NOT NULL,
	event_type text NOT NULL,
	causation_id uuid NULL,
	correlation_id uuid NULL,
	data jsonb NOT NULL,
	metadata jsonb NULL,
	created_at timestamp WITH time zone DEFAULT NOW(
) NOT NULL
);

CREATE TABLE stream_events (
	event_id uuid NOT NULL REFERENCES events (event_id),
	stream_id bigint NOT NULL REFERENCES streams (stream_id),
	stream_version bigint NOT NULL,
	original_stream_id bigint REFERENCES streams (stream_id),
	original_stream_version bigint,
	PRIMARY KEY (event_id, stream_id)
);

CREATE UNIQUE INDEX ix_stream_events ON stream_events (stream_id, stream_version);
