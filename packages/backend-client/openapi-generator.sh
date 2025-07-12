#!/usr/bin/env bash

API_DOCS_SOURCE=$1
SERVER=$2
API_DOCS_PATH="swagger/doc.json"
API_DOCS_FILE="./api-docs.json"

function fetch_api_docs() {
  echo "🌎 Fetching api docs from $SERVER/$API_DOCS_PATH"
  curl "$SERVER/$API_DOCS_PATH" > $API_DOCS_FILE 2> /dev/null
  exit_on_error "Error during polling. Has the server been started?"
}

function clean_up() {
    echo "🧹 Cleaning up src directory..."
    rm -rf src/
    mkdir src/

    exit_on_error "Could not delete the src folder"
}

function generate_client_docker() {
  docker run --rm -v "$PWD":/local -u "$(id -u)":"$(id -g)" openapitools/openapi-generator-cli:v7.14.0 generate \
    -i /local/$API_DOCS_FILE \
    -o /local/src/ \
    -g typescript-fetch \
    --language-specific-primitives \
    --reserved-words-mappings \
    --additional-properties=supportsES6=true

  exit_on_error "Could not create the client"
}

function generate_client() {
  openapi-generator-cli generate \
    -i $API_DOCS_FILE \
    -o src \
    -g typescript-fetch \
    --language-specific-primitives \
    --reserved-words-mappings \
    --additional-properties=supportsES6=true

  exit_on_error "Could not create the client"
}

function revise_generation() {
    echo "🪄 Revision of the generated client"

    # Nothing to do here for now
    echo " Nothing to do here for now"
}

function update_packages_version () {
  VERSION=$(yq -r '.info.version' api-docs.json)
  echo "📝 Update version in packages.json to version $VERSION"
  version="$VERSION" yq -i '.version=env(version)' package.json
}

function exit_on_error() {
  if [ $? -ne 0 ]; then
    echo "❌ $1"
    exit 1
  fi
}

function main() {
  if [ -z "$API_DOCS_SOURCE" ]; then
    echo "❌ Please provide the API docs source"
    exit 1
  elif [ "$API_DOCS_SOURCE" == "local" ]; then
    echo "📚 Using local API docs"
  elif [ "$API_DOCS_SOURCE" == "remote" ]; then
    echo "🌍 Using remote API docs from $SERVER"
    fetch_api_docs
  else
    echo "❌ Invalid API docs source"
    exit 1
  fi

  clean_up
  if [ "$API_DOCS_SOURCE" == "local" ]; then
    echo "Generating client"
    generate_client
  else
    echo "🐳 Generating client using docker"
    generate_client_docker
  fi
  revise_generation

  update_packages_version

  echo "✨ Done"
}

main

