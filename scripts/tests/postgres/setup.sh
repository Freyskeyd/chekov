#!/bin/bash

set -e
set -u

function create_user_and_database() {
	local database=$1
	echo "  Creating user and database '$database'"
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
	    CREATE USER $database;
	    CREATE DATABASE $database;
	    GRANT ALL PRIVILEGES ON DATABASE $database TO $database;
EOSQL
}

create_user_and_database "event_store_bank"
create_user_and_database "bank"
create_user_and_database "event_store_gift_shop"
create_user_and_database "gift_shop"

psql -v ON_ERROR_STOP=1 --dbname=event_store_bank --username="event_store_bank" -f /tmp/setup_event_store.sql
psql -v ON_ERROR_STOP=1 --dbname=bank --username="bank" -f /tmp/setup_bank.sql

psql -v ON_ERROR_STOP=1 --dbname=event_store_gift_shop --username="event_store_gift_shop" -f /tmp/setup_event_store.sql
psql -v ON_ERROR_STOP=1 --dbname=gift_shop  --username="gift_shop" -f /tmp/setup_gift_shop.sql

