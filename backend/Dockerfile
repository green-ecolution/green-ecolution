FROM lukemathwalker/cargo-chef:latest-rust-1.98@sha256:b8cfeff0a392bc4bf5066210875728562b5b6ba66fe3b67aae0c6b7d56df1043 AS chef
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

FROM gcr.io/distroless/cc-debian13:nonroot@sha256:54df941ed0d06a1bd95ef5e0ce391fd8d9f94b64782dc9a60062727849ee3f97 AS runtime
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

COPY --from=builder --chown=10001:10001 /usr/local/bin/green-ecolution green-ecolution
COPY --from=builder --chown=10001:10001 /usr/local/bin/migrate migrate
COPY --chown=10001:10001 config config

USER 10001:10001
EXPOSE 3030

ENV APP_ENVIRONMENT=production

# Exec form: the distroless runtime has no shell. The binary resolves the port
# from the same config the server uses, so it follows a port override.
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD ["./green-ecolution", "healthcheck"]

ENTRYPOINT ["./green-ecolution"]
