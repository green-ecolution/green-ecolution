#!/usr/bin/env bash

action=$1

if [ "$action" == "export" ]; then
    docker compose exec -u root -it keycloak kc.sh export --dir=/opt/bitnami/keycloak/data/import --realm green-ecolution --users=realm_file --optimized
elif [ "$action" == "import" ]; then
    docker compose exec -u root -it keycloak kc.sh import --file=/opt/bitnami/keycloak/data/import/green-ecolution-realm.json --optimized
else
    echo "Usage: $0 export|import"
    exit 1
fi


