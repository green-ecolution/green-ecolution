#############################################
# base
#############################################
FROM node:26-alpine AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN npm install -g pnpm
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
ARG VITE_AUTH_BYPASS="false"
ARG VITE_OIDC_AUTHORITY=""
ARG VITE_OIDC_CLIENT_ID="frontend"
RUN if [ "$ENV" == "dev" ]; then \
        APP_VERSION="$APP_VERSION" VITE_BACKEND_BASEURL="$BASEURL" VITE_AUTH_BYPASS="$VITE_AUTH_BYPASS" VITE_OIDC_AUTHORITY="$VITE_OIDC_AUTHORITY" VITE_OIDC_CLIENT_ID="$VITE_OIDC_CLIENT_ID" pnpm build:stage; \
    else \
        APP_VERSION="$APP_VERSION" VITE_BACKEND_BASEURL="$BASEURL" VITE_AUTH_BYPASS="$VITE_AUTH_BYPASS" VITE_OIDC_AUTHORITY="$VITE_OIDC_AUTHORITY" VITE_OIDC_CLIENT_ID="$VITE_OIDC_CLIENT_ID" pnpm build; \
    fi

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
