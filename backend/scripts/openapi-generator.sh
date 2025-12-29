#!/usr/bin/env bash

PACKAGE_NAME="$1"
API_DOCS_FILE="$2"
API_DOCS_OUT_DIR="$3"

generate_client_docker() {
  rm -rf $API_DOCS_OUT_DIR

  docker run --rm -v "${PWD}":/local -u "$(id -u):$(id -g)" openapitools/openapi-generator-cli:v7.14.0 generate \
    -i /local/$API_DOCS_FILE \
    -o /local/$API_DOCS_OUT_DIR \
    -g go \
    --package-name $PACKAGE_NAME \
    --git-host github.com \
    --git-user-id green-ecolution \
    --git-repo-id green-ecolution/backend/pkg \
    --skip-overwrite \
    --additional-properties=isGoSubmodule=true

  exit_on_error "Could not create the client"

  cd $API_DOCS_OUT_DIR
}

exit_on_error() {
  if [ $? -ne 0 ]; then
    echo "❌ $1"
    exit 1
  fi
}

generate_client_docker
