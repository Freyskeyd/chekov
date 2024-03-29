CREATE TABLE streams (
	stream_id bigserial primary key,
	stream_uuid text NOT NULL,
	stream_version bigint DEFAULT 0 NOT NULL,
	created_at timestamp WITH time zone DEFAULT NOW(
) NOT NULL,
	deleted_at timestamp WITH time zone
);


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
	stream_id bigserial NOT NULL REFERENCES streams (stream_id),
	stream_version bigint NOT NULL,
	original_stream_id bigserial REFERENCES streams (stream_id),
	original_stream_version bigint,
	PRIMARY KEY (event_id, stream_id)
);


CREATE OR REPLACE FUNCTION notify_events()
	RETURNS trigger AS $$
DECLARE
	payload text;
BEGIN
	-- Payload text contains:
	--  * `stream_uuid`
	--  * `stream_id`
	--  * first `stream_version`
	--  * last `stream_version`
	-- Each separated by a comma (e.g. 'stream-12345,1,1,5')
	payload := NEW.stream_uuid || ',' || NEW.stream_id || ',' || (OLD.stream_version + 1) || ',' || NEW.stream_version;
	-- Notify events to listeners
	PERFORM pg_notify('events', payload);
	RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION create_global_stream_events()
	RETURNS trigger AS $$
DECLARE
	all_stream streams%ROWTYPE;
BEGIN
	UPDATE streams SET stream_version = stream_version + 1 WHERE stream_id = 0 RETURNING * INTO all_stream;
	INSERT INTO stream_events (event_id, stream_id, stream_version, original_stream_id, original_stream_version) VALUES (NEW.event_id, 0, all_stream.stream_version, NEW.stream_id, NEW.stream_version);

	RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER event_notification AFTER UPDATE ON streams FOR EACH ROW EXECUTE PROCEDURE notify_events();
CREATE TRIGGER propagate_stream_events AFTER INSERT ON stream_events FOR EACH ROW WHEN (NEW.stream_id != 0 ) EXECUTE PROCEDURE create_global_stream_events();

/* CREATE UNIQUE INDEX ix_subscriptions_stream_uuid_subscription_name ON subscriptions (stream_uuid, subscription_name); */
CREATE UNIQUE INDEX ix_stream_events ON stream_events (stream_id, stream_version);
CREATE UNIQUE INDEX ix_streams_stream_uuid ON streams (stream_uuid);

CREATE RULE no_update_stream_events AS ON UPDATE TO stream_events DO INSTEAD NOTHING;
CREATE RULE no_delete_stream_events AS ON DELETE TO stream_events DO INSTEAD NOTHING;
CREATE RULE no_update_events AS ON UPDATE TO events DO INSTEAD NOTHING;
CREATE RULE no_delete_events AS ON DELETE TO events DO INSTEAD NOTHING;

INSERT INTO streams (stream_id, stream_uuid, stream_version) VALUES (0, '$all', 0);
