#############################################
# base
#############################################
FROM node:26-alpine AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN npm install -g pnpm@10.20.0
# keycloakify builds the jar by shelling out to `mvn clean install`
RUN apk add --no-cache maven openjdk21-jdk
COPY . /app
WORKDIR /app

#############################################
# Builder theme
#############################################
FROM base AS build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm --filter @green-ecolution/keycloak-theme run build-keycloak-theme

#############################################
# Keycloak
#############################################
FROM quay.io/keycloak/keycloak:26.7 AS runner
COPY --from=build /app/keycloak-theme/dist_keycloak/keycloak-theme.jar /opt/keycloak/providers/
RUN /opt/keycloak/bin/kc.sh build
