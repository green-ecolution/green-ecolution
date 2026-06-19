# syntax=docker/dockerfile:1

#############################################
# Stage 1: domain WASM bindings (wasm-pack)
#############################################
FROM rust:1.96-bookworm AS wasm
RUN rustup target add wasm32-unknown-unknown \
  && curl -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh
WORKDIR /repo
COPY backend-rs/ backend-rs/
RUN wasm-pack build backend-rs/crates/domain-wasm --target bundler \
      --out-dir /out/pkg --release

#############################################
# Stage 2: frontend (Vite) build
#############################################
FROM node:26-alpine AS frontend
ENV PNPM_HOME="/pnpm" PATH="/pnpm:$PATH"
RUN npm install -g pnpm
WORKDIR /repo/frontend
COPY frontend/ .
COPY --from=wasm /out/pkg packages/domain-wasm/pkg
ARG BASEURL="/api"
ARG APP_VERSION="dev"
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN APP_VERSION="$APP_VERSION" VITE_BACKEND_BASEURL="$BASEURL" pnpm build

#############################################
# Stage 3: cargo-chef planner
#############################################
FROM lukemathwalker/cargo-chef:latest-rust-1.96@sha256:e606721f52d95169364bf39cae726a94ed8b397625011ccfaa8340db488b823b AS chef
WORKDIR /repo/backend-rs

FROM chef AS planner
COPY backend-rs/ .
RUN cargo chef prepare --recipe-path recipe.json

#############################################
# Stage 4: rust builder (embeds the SPA)
#############################################
FROM chef AS builder
RUN apt-get update && apt-get install -y --no-install-recommends lld clang \
  && rm -rf /var/lib/apt/lists/*

COPY --from=planner /repo/backend-rs/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/repo/backend-rs/target,sharing=locked \
    cargo chef cook --release --locked --recipe-path recipe.json

COPY backend-rs/ .
# rust-embed folder is `../../../frontend/app/dist` relative to crates/server,
# i.e. /repo/frontend/app/dist — copy the built SPA there before compiling.
COPY --from=frontend /repo/frontend/app/dist /repo/frontend/app/dist

ARG GE_GIT_COMMIT=unknown
ARG GE_GIT_BRANCH=unknown
ARG GE_BUILD_TIME=unknown

ENV SQLX_OFFLINE=true \
    GE_GIT_COMMIT=$GE_GIT_COMMIT \
    GE_GIT_BRANCH=$GE_GIT_BRANCH \
    GE_BUILD_TIME=$GE_BUILD_TIME

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/repo/backend-rs/target,sharing=locked \
    cargo build --release --locked --features embed-frontend --bin green-ecolution --bin migrate \
    && cp target/release/green-ecolution /usr/local/bin/green-ecolution \
    && cp target/release/migrate /usr/local/bin/migrate

#############################################
# Stage 5: runtime
#############################################
FROM debian:trixie-slim@sha256:b6e2a152f22a40ff69d92cb397223c906017e1391a73c952b588e51af8883bf8 AS runtime
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
COPY --chown=10001:10001 backend-rs/config config

USER 10001:10001
EXPOSE 3020

ENV APP_ENVIRONMENT=production

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD wget --quiet --tries=1 --spider http://localhost:3020/health || exit 1

ENTRYPOINT ["/usr/bin/tini", "--", "./green-ecolution"]
