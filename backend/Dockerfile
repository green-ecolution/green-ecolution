FROM lukemathwalker/cargo-chef:latest-rust-1.97@sha256:7341ca3ca2726f3249fb227b510ce195fb80d6e584ff8894b68ffa97f1512e62 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y --no-install-recommends lld clang \
  && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo chef cook --release --locked --recipe-path recipe.json

COPY . .

ARG GE_GIT_COMMIT=unknown
ARG GE_GIT_BRANCH=unknown
ARG GE_BUILD_TIME=unknown

ENV SQLX_OFFLINE=true \
    GE_GIT_COMMIT=$GE_GIT_COMMIT \
    GE_GIT_BRANCH=$GE_GIT_BRANCH \
    GE_BUILD_TIME=$GE_BUILD_TIME

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --locked --bin green-ecolution --bin migrate \
    && cp target/release/green-ecolution /usr/local/bin/green-ecolution \
    && cp target/release/migrate /usr/local/bin/migrate

FROM debian:trixie-slim@sha256:020c0d20b9880058cbe785a9db107156c3c75c2ac944a6aa7ab59f2add76a7bd AS runtime
WORKDIR /app

ARG APP_VERSION=dev
ARG APP_REVISION=unknown

LABEL org.opencontainers.image.title="green-ecolution" \
      org.opencontainers.image.description="Smart irrigation and green-space management platform for municipal urban greenery." \
      org.opencontainers.image.source="https://github.com/green-ecolution/green-ecolution" \
      org.opencontainers.image.documentation="https://github.com/green-ecolution/green-ecolution#readme" \
      org.opencontainers.image.url="https://green-ecolution.de" \
      org.opencontainers.image.vendor="Green Ecolution" \
      org.opencontainers.image.licenses="AGPL-3.0-or-later" \
      org.opencontainers.image.version="${APP_VERSION}" \
      org.opencontainers.image.revision="${APP_REVISION}"

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends ca-certificates tini wget \
  && rm -rf /var/lib/apt/lists/* \
  && groupadd --gid 10001 app \
  && useradd --uid 10001 --gid app --no-create-home --shell /usr/sbin/nologin app

COPY --from=builder --chown=10001:10001 /usr/local/bin/green-ecolution green-ecolution
COPY --from=builder --chown=10001:10001 /usr/local/bin/migrate migrate
COPY --chown=10001:10001 config config

USER 10001:10001
EXPOSE 3020

ENV APP_ENVIRONMENT=production

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD wget --quiet --tries=1 --spider http://localhost:3020/api/health || exit 1

ENTRYPOINT ["/usr/bin/tini", "--", "./green-ecolution"]
