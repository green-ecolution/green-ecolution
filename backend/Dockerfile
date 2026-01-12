#############################################
# Preparer go
#############################################
FROM golang:1.25-alpine AS preparer_go

ARG MOCKER_VERSION="v2.52.2"

WORKDIR /app/build

# install build dependencies
COPY ./go.mod ./go.sum ./
RUN apk add --no-cache make git geos-dev build-base proj-dev
RUN go mod download

COPY . .

#############################################
# Builder go
#############################################
FROM preparer_go AS builder

ARG APP_VERSION="v0.0.0"
ARG APP_GIT_COMMIT="unknown"
ARG APP_GIT_BRANCH="develop"
ARG APP_GIT_REPOSITORY="https://github.com/green-ecolution/green-ecolution/backend"
ARG APP_BUILD_TIME="unknown"

RUN CGO_ENABLED=1 go build -o "bin/green-ecolution" \
    -ldflags=" \
      -s -w \
      -X main.version=${APP_VERSION} \
      -X github.com/green-ecolution/green-ecolution/backend/internal/storage/local/info.version=${APP_VERSION} \
      -X github.com/green-ecolution/green-ecolution/backend/internal/storage/local/info.gitCommit=${APP_GIT_COMMIT} \
      -X github.com/green-ecolution/green-ecolution/backend/internal/storage/local/info.gitBranch=${APP_GIT_BRANCH} \
      -X github.com/green-ecolution/green-ecolution/backend/internal/storage/local/info.gitRepository=${APP_GIT_REPOSITORY} \
      -X github.com/green-ecolution/green-ecolution/backend/internal/storage/local/info.buildTime=${APP_BUILD_TIME} \
    "

#############################################
# Runner go
#############################################
FROM alpine:3.23 AS runner

EXPOSE 3000

RUN adduser -D gorunner
RUN apk add --no-cache geos proj curl

USER gorunner
WORKDIR /app

COPY --chown=gorunner:gorunner --from=builder /app/build/bin/green-ecolution /app/backend

ENTRYPOINT [ "/app/backend" ]
