# Demo plugin

A complete, deliberately small Green Ecolution plugin for manually testing the plugin
integration. It covers both halves: its own view, which runs inside the application's
iframe and uses `@green-ecolution/plugin-interface`, and importing data through the
ingest API.

The container holds two parts: the built view and a small Node server with no third-party
dependencies. The server serves the view, keeps the API key in memory and makes the ingest
calls itself, because the key must never reach the browser.

## Running it

```bash
just plugin-demo-up     # builds the image and starts the container
just plugin-demo-down   # stops and removes it again
```

The view then lives at <http://localhost:5175>. Opened directly in a browser it only shows
a notice: without the handshake with the application there is no context.

The API base url comes from `GE_API_BASE_URL`. The default
`http://host.docker.internal:3030/api` matches `just run-dev`; in the full docker stack
`compose.app.yaml` overrides it with `http://backend:3000/api`.

## Manual test

1. Start the infrastructure and the application (`just infra-up`, `just run-dev`), then
   `just plugin-demo-up`.
2. In the application go to Settings → Plugins → install a plugin:
   - slug `demo-plugin`, name `Demo-Plugin`
   - organization: your own
   - the plugin's own permissions: `tree:create`, `tree:update`, `tree:delete`
   - permission required to open the view: `tree:read`
   - frontend: `external` with `http://localhost:5175`
3. Copy the API key shown once on installation.
4. Enable the plugin; a newly installed plugin is deliberately disabled.
5. Open the view. It shows the signed-in person's name, the slug, the language and the
   appearance, which means the handshake worked.
6. Paste the key. The view then shows the identity the backend returns from
   `/plugins/me`.
7. Run the import: six trees, answer `created: 6`. Run it again: `unchanged: 6`.
8. Run the modification: `updated: 1`. Tree `DEMO-003` now carries the species
   `Tilia tomentosa` in the tree list.
9. Run the deletion: `204`. `DEMO-006` is gone from the tree list.
10. List the references: five `external_id` to `tree_id` mappings.
11. Clean up: uninstall the plugin, then `just plugin-demo-down`. The imported trees stay;
    uninstalling only removes the registration.

## Limits that are intentional

- Pasting the key sends it to the demo plugin's own server over plain HTTP once. That is
  acceptable for a demo on `localhost` but not for a real plugin, where the key belongs in
  the adapter's configuration rather than in a form.
- The key lives in memory only. Restarting the container forgets it.
- The view reports its height with `notifyResize`. The application currently ignores that,
  because the viewer route gives the iframe a fixed height.
- `PluginProvider` renders nothing while the handshake is in flight, so a plugin needs its
  own placeholder. Here it sits statically in `index.html` and is removed on mount.
