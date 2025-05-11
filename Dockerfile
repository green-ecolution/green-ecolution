#############################################
# Preparer go
#############################################
FROM golang:1.24-alpine AS preparer_go

ARG MOCKER_VERSION="v2.52.2"

WORKDIR /app/build

# install build dependencies
COPY ./Makefile ./go.mod ./go.sum ./
RUN apk add --no-cache make git geos-dev build-base proj-dev
RUN make setup

COPY . .

#############################################
# Builder go
#############################################
FROM preparer_go AS builder

ARG APP_VERSION="v0.0.0"
ARG APP_GIT_COMMIT="unknown"
ARG APP_GIT_BRANCH="develop"
ARG APP_GIT_REPOSITORY="https://github.com/green-ecolution/backend"
ARG APP_BUILD_TIME="unknown"

RUN make build \
    APP_VERSION=${APP_VERSION} \
    APP_GIT_COMMIT=${APP_GIT_COMMIT} \
    APP_GIT_BRANCH=${APP_GIT_BRANCH} \
    APP_GIT_REPOSITORY=${APP_GIT_REPOSITORY} \
    APP_BUILD_TIME=${APP_BUILD_TIME}

#############################################
# Runner go
#############################################
FROM alpine:3.21 AS runner

EXPOSE 3000
ARG ENV="stage"

RUN adduser -D gorunner
RUN apk add --no-cache geos proj curl

USER gorunner
WORKDIR /app

COPY --chown=gorunner:gorunner --from=builder /app/build/config/config.default.yaml /app/config/config.${ENV}.yaml
COPY --chown=gorunner:gorunner --from=builder /app/build/bin/green-ecolution-backend /app/backend

ENTRYPOINT [ "/app/backend" ]
