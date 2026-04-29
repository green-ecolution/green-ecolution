FROM lukemathwalker/cargo-chef:latest-rust-1.94 AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends lld clang \
  && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --locked --bin green-ecolution --bin migrate

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends ca-certificates \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/* \
  && groupadd --gid 10001 app \
  && useradd --uid 10001 --gid app --no-create-home --shell /usr/sbin/nologin app

COPY --from=builder --chown=app:app /app/target/release/green-ecolution green-ecolution
COPY --from=builder --chown=app:app /app/target/release/migrate migrate
COPY --chown=app:app config config

USER app
EXPOSE 3020

ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./green-ecolution"]
