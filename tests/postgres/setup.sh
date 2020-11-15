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

create_user_and_database "event_store"
create_user_and_database "bank"

psql -v ON_ERROR_STOP=1 --dbname=event_store --username "$POSTGRES_USER" -f /docker-entrypoint-initdb.d/setup_event_store.sql
psql -v ON_ERROR_STOP=1 --dbname=bank --username "$POSTGRES_USER" -f /docker-entrypoint-initdb.d/setup_bank.sql
