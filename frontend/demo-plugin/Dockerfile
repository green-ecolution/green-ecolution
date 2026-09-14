#############################################
# build
#############################################
FROM node:26-alpine AS build
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN npm install -g pnpm@10.20.0
COPY . /app
WORKDIR /app
# The trailing dots pull in plugin-interface as well, and nothing else: the app
# and its wasm package are not needed to build the demo plugin.
RUN --mount=type=cache,id=pnpm,target=/pnpm/store \
    pnpm install --frozen-lockfile --filter @green-ecolution/demo-plugin...
RUN pnpm --filter @green-ecolution/demo-plugin build

#############################################
# runner
#############################################
FROM node:26-alpine AS runner
WORKDIR /srv
COPY --from=build /app/demo-plugin/dist ./dist
COPY --from=build /app/demo-plugin/server ./server
ENV PORT=80
EXPOSE 80
CMD ["node", "server/server.mjs"]
