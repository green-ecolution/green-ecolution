FROM lukemathwalker/cargo-chef:latest-rust-1.98@sha256:94a624f8475651c2b56281b29936cab5cbe21c431677880b6a98d47561fa3289 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y --no-install-recommends lld clang \
  && rm -rf /var/lib/apt/lists/*

# Deliberately no --mount=type=cache here. BuildKit cache exporters (gha,
# registry) export layer diffs, never cache mounts, so a mounted /app/target
# left this layer empty and CI recompiled the whole dependency tree every run.
# Writing into the layer is what makes cargo-chef work across machines.
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json

COPY . .

ARG GE_GIT_COMMIT=unknown
ARG GE_GIT_BRANCH=unknown
ARG GE_BUILD_TIME=unknown

ENV SQLX_OFFLINE=true \
    GE_GIT_COMMIT=$GE_GIT_COMMIT \
    GE_GIT_BRANCH=$GE_GIT_BRANCH \
    GE_BUILD_TIME=$GE_BUILD_TIME

RUN cargo build --release --locked --bin green-ecolution --bin migrate \
    && cp target/release/green-ecolution /usr/local/bin/green-ecolution \
    && cp target/release/migrate /usr/local/bin/migrate

FROM gcr.io/distroless/cc-debian13:nonroot@sha256:e792ab3d241a468a4fd7519ddbbebe66b49b5f365771716ea688ad40b6c6f1c2 AS runtime
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
