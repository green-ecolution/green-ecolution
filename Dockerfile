#############################################
# Builder web
#############################################
FROM openapitools/openapi-generator-cli:v7.14.0 AS openapi-builder

USER root
RUN echo "java -jar /opt/openapi-generator/modules/openapi-generator-cli/target/openapi-generator-cli.jar \$@" > /usr/local/bin/openapi-generator-cli \
    && chmod +x /usr/local/bin/openapi-generator-cli

WORKDIR /app
COPY ./backend-client/api-docs.json ./backend-client/openapi-generator.sh ./
RUN ./openapi-generator.sh local

#############################################
# base
#############################################
FROM node:24-alpine AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY . /app
COPY --from=openapi-builder /app/src /app/backend-client/src
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

COPY --from=build /app/frontend/dist /usr/share/nginx/html
