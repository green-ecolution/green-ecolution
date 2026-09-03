#############################################
# base
#############################################
FROM node:26-alpine AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN npm install -g pnpm@10.20.0
COPY . /app
WORKDIR /app

#############################################
# Builder web
#############################################
FROM base AS build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile

ARG ENV="prod"
ARG BASEURL="/api"
ARG APP_VERSION=""
RUN if [ "$ENV" == "dev" ]; then \
        APP_VERSION="$APP_VERSION" VITE_BACKEND_BASEURL="$BASEURL" pnpm build:stage; \
    else \
        APP_VERSION="$APP_VERSION" VITE_BACKEND_BASEURL="$BASEURL" pnpm build; \
    fi

#############################################
# Handbook PDF
#############################################
FROM alpine:3.22 AS handbook
# amd64-only pin; add an arch mapping here if another platform target is ever built.
# 0.14.2 is the version the handbook layout was verified against; the local `just`
# recipe and the CI job must stay on the same pin. TYPST_VERSION and TYPST_SHA256
# describe one release each and must be updated together.
ARG TYPST_VERSION=0.14.2
ARG TYPST_SHA256=a6044cbad2a954deb921167e257e120ac0a16b20339ec01121194ff9d394996d
RUN apk add --no-cache curl tar xz \
    && curl -fsSL -o /tmp/typst.tar.xz \
        "https://github.com/typst/typst/releases/download/v${TYPST_VERSION}/typst-x86_64-unknown-linux-musl.tar.xz" \
    && echo "${TYPST_SHA256}  /tmp/typst.tar.xz" | sha256sum -c - \
    && tar -xJf /tmp/typst.tar.xz -C /tmp \
    && mv /tmp/typst-x86_64-unknown-linux-musl/typst /usr/local/bin/typst \
    && rm -rf /tmp/typst.tar.xz /tmp/typst-x86_64-unknown-linux-musl

COPY --from=build /app/handbook /app/handbook
COPY --from=build /app/app/public/fonts /app/app/public/fonts

RUN mkdir -p /out && typst compile \
        --root /app/handbook \
        --font-path /app/handbook/typst/fonts \
        --font-path /app/app/public/fonts \
        /app/handbook/typst/manual.typ \
        /out/green-ecolution-handbuch.pdf

#############################################
# Nginx
#############################################
FROM nginx:1.31 AS runner
RUN rm -rf /etc/nginx/conf.d/default.conf && cat <<EOF > /etc/nginx/conf.d/nginx.conf
server {
    listen       80;
    listen  [::]:80;

    location /status {
        stub_status on;
        access_log   off;
    }

    location / {
        root   /usr/share/nginx/html;
        try_files \$uri \$uri/ /index.html;
    }
}
EOF

COPY --from=build /app/app/dist /usr/share/nginx/html
COPY --from=handbook /out/green-ecolution-handbuch.pdf /usr/share/nginx/html/handbook/
