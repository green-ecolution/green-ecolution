#!/usr/bin/env bash
# Restore a staging pg_dump into the local dev database and adapt it to the dev
# setup (see scripts/staging-to-dev.sql). Destructive: schema public is dropped.
#
# Usage: scripts/import-staging-dump.sh [dump.sql] [--yes]
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dump="${1:-staging.sql}"
assume_yes="${ASSUME_YES:-0}"

for arg in "$@"; do
  case "$arg" in
    -y|--yes) assume_yes=1 ;;
  esac
done
[[ "${dump}" == -* ]] && dump="staging.sql"

pg_user="${POSTGRES_USER:-postgres}"
pg_password="${POSTGRES_PASSWORD:-postgres}"
pg_db="${POSTGRES_DB:-postgres}"
pg_host="${POSTGRES_HOST:-localhost}"
pg_port="${POSTGRES_PORT:-5432}"

seed_orgs="${repo_root}/backend/seeds/01_organizations.sql"
overlay="${repo_root}/scripts/staging-to-dev.sql"

for f in "$dump" "$seed_orgs" "$overlay"; do
  [[ -f "$f" ]] || { echo "error: missing $f" >&2; exit 1; }
done

# Host psql if there is one, otherwise the psql inside the db container.
if command -v psql >/dev/null 2>&1; then
  psql_mode="host"
  run_psql() { PGPASSWORD="$pg_password" psql -qtAX -v ON_ERROR_STOP=1 \
    -h "$pg_host" -p "$pg_port" -U "$pg_user" -d "$pg_db" "$@"; }
else
  psql_mode="container"
  run_psql() { docker compose -f "${repo_root}/compose.yaml" exec -T db \
    env PGPASSWORD="$pg_password" psql -qtAX -v ON_ERROR_STOP=1 \
    -h localhost -U "$pg_user" -d "$pg_db" "$@"; }
fi

# pg_dump 18 emits psql meta-commands a psql 17 client rejects, and the dump
# carries ownership/grants for a role that only exists on staging.
filter_dump() {
  grep -vE '^\\(un)?restrict [A-Za-z0-9]+$|^ALTER [A-Z][A-Z ]* .* OWNER TO |^GRANT |^REVOKE |^COMMENT ON EXTENSION '
}

echo "Importing $(du -h "$dump" | cut -f1) dump '$dump' into ${pg_user}@${pg_host}:${pg_port}/${pg_db} (psql: ${psql_mode})"

if [[ "$assume_yes" != "1" ]]; then
  read -r -p "This DROPs schema public and replaces all local data. Continue? [y/N] " reply </dev/tty
  [[ "$reply" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 1; }
fi

dump_migration="$(grep -oE '^[0-9]{14}\b' "$dump" | sort | tail -n1 || true)"
local_migration="$(ls "${repo_root}/backend/migrations" | grep -oE '^[0-9]{14}' | sort | tail -n1)"
if [[ -n "$dump_migration" && "$dump_migration" != "$local_migration" ]]; then
  echo "warning: dump is at migration ${dump_migration}, repo at ${local_migration} — run 'just migrate-up' afterwards" >&2
fi

echo "Dropping schema public..."
echo 'DROP SCHEMA IF EXISTS public CASCADE; CREATE SCHEMA public;' | run_psql -f -

echo "Restoring dump..."
filter_dump < "$dump" | run_psql -f -

echo "Applying dev overlay..."
{ echo 'BEGIN;'; cat "$seed_orgs"; cat "$overlay"; echo 'COMMIT;'; } | run_psql -f -

echo
echo "Resources per organization:"
run_psql -c "
  SELECT o.name,
         (SELECT count(*) FROM tree_clusters  x WHERE x.organization_id = o.id) AS clusters,
         (SELECT count(*) FROM trees          x WHERE x.organization_id = o.id) AS trees,
         (SELECT count(*) FROM sensors        x WHERE x.organization_id = o.id) AS sensors,
         (SELECT count(*) FROM vehicles       x WHERE x.organization_id = o.id) AS vehicles,
         (SELECT count(*) FROM depots         x WHERE x.organization_id = o.id) AS depots,
         (SELECT count(*) FROM watering_plans x WHERE x.organization_id = o.id) AS plans,
         (SELECT count(*) FROM user_profiles  x WHERE x.organization_id = o.id) AS users
    FROM organizations o
   ORDER BY o.name;" | column -t -s '|'

echo
echo "Done. Log in with ge.admin / tbz.* / externa.* / externb.* (username = password)."
