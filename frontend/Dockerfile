#############################################
# base
#############################################
FROM node:24-alpine AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY . /app
WORKDIR /app

#############################################
# Builder web
#############################################
FROM base AS build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile

ARG ENV="prod"
ARG BASEURL="/api"
RUN if [ "$ENV" == "dev" ]; then \
        VITE_BACKEND_BASEURL="$BASEURL" pnpm build:stage; \
    else \
        VITE_BACKEND_BASEURL="$BASEURL" pnpm build; \
    fi

#############################################
# Nginx
#############################################
FROM nginx:1.29 AS runner
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
