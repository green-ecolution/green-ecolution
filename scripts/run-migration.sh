#!/bin/bash

set -e
MIGRATION_WITH_SEED=${MIGRATION_WITH_SEED:-false}
MIGRATION_WITH_DB_RESET=${MIGRATION_WITH_DB_RESET:-false}
migration_dir=$1
seed_dir=$2

if [ -v $migration_dir && -v $seed_dir]; then
  echo "Usage: $0 <migration_dir> <seed_dir>"
  echo "  Example: $0 /migrations /seed"
  exit 1
fi

# Create goose_db if not present
goose -dir /migrations postgres "postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@$POSTGRES_HOST:$POSTGRES_PORT/$POSTGRES_DB" status

if [ "$MIGRATION_WITH_DB_RESET" = "true" ]; then
  echo "Resetting database"
  goose -dir /migrations postgres "postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@$POSTGRES_HOST:$POSTGRES_PORT/$POSTGRES_DB" reset
fi

echo "Running migration"
goose -dir /migrations postgres "postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@$POSTGRES_HOST:$POSTGRES_PORT/$POSTGRES_DB" up

if [ "$MIGRATION_WITH_SEED" = "false" ]; then
    echo "Migration completed"
    exit 0
fi

echo "Applying seed"
goose -dir /seed -no-versioning postgres "postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@$POSTGRES_HOST:$POSTGRES_PORT/$POSTGRES_DB" up
