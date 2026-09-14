# @green-ecolution/plugin-interface

SDK for building a Green Ecolution plugin: the browser-side handshake between the host
application and a plugin's iframe, plus a small React helper around it.

A Green Ecolution plugin is an external system, installed by an administrator through
the application's settings, that may do either or both of two things: write data into
Green Ecolution through its ingest API, and contribute its own view, embedded in an
iframe inside the application. This package covers the second half, the view. The
first half, writing data, is a plain HTTP contract described below and needs no SDK at
all.

There is no Module Federation here, no remote bundle loading, and no shared React
runtime between host and plugin. A plugin is an ordinary web application that happens
to run inside an iframe and speaks one small `postMessage` protocol to its host.

A complete working plugin using this SDK lives in `frontend/demo-plugin`: it renders the
context from the handshake and imports demo trees through the ingest contract described
below.

## Installation

```bash
pnpm add @green-ecolution/plugin-interface
```

React 19 is a peer dependency; install `react` and `react-dom` alongside it if your
plugin does not already depend on them.

## How a plugin is registered

A plugin never registers itself. An administrator creates it from **Settings → Plugins**
in Green Ecolution, choosing a slug, the organization it belongs to, the permissions it
receives, and, if it has a view, where that view lives (an external `https` URL, or an
in-cluster service address the operator proxies). Installing issues a plaintext API key
exactly once; that key is what the plugin's own backend adapter uses to call the ingest
endpoints below. There is no self-service registration call to make from plugin code.

If your plugin only pushes data and has no view, you can stop reading here and go
straight to the ingest contract.

## The view: embedding and the handshake

When a plugin has a view, Green Ecolution renders it in a sandboxed iframe
(`allow-scripts allow-forms allow-popups allow-same-origin`, `referrerPolicy="no-referrer"`,
no `allow` features) and waits for it to say hello. The protocol is a small envelope
carried over `window.postMessage`:

```typescript
type Envelope<T> = {
  ns: 'green-ecolution'
  v: 1
  type: string
  payload: T
}
```

Three message types make up the current protocol:

- `ge:hello` — sent by the plugin to its parent window on load, empty payload.
- `ge:init` — sent by the host in reply, carrying the [`PluginContext`](#plugincontext).
- `ge:resize` — sent by the plugin whenever its content height changes, `{ height: number }`.

Both sides check more than the message shape before trusting it: the host only replies
to a `ge:hello` whose `event.source` is the iframe's own `contentWindow` (checking
`origin` alone would accept a forged message from any other same-origin frame on the
host page), and the plugin only accepts a `ge:init` whose `event.source` is
`window.parent`. None of this carries a secret, so the target origin on the plugin's
side is `'*'`; the check is about which window sent the message, not about hiding its
contents.

### `connectToHost()`

Call this once, as early as possible, from inside your plugin's own document. It sends
`ge:hello` and resolves with the [`PluginContext`](#plugincontext) once the host answers:

```typescript
import { connectToHost } from '@green-ecolution/plugin-interface'

const context = await connectToHost()
```

The hello is repeated every 250 ms until the host answers, because the host attaches its
listener from an effect and a single hello can arrive before it is listening. After
10 seconds without an answer the promise rejects rather than hanging, which is what you
see when the document is opened directly instead of embedded as a plugin view. Both
values are adjustable, and `0` disables either behaviour:

```typescript
const context = await connectToHost({ retryIntervalMs: 100, timeoutMs: 0 })
```

### `notifyResize(height)`

Call this whenever your content's height changes, so a host that acts on it can size
the iframe accordingly:

```typescript
import { notifyResize } from '@green-ecolution/plugin-interface'

notifyResize(document.documentElement.scrollHeight)
```

### `PluginProvider` and `usePluginContext`

For a React plugin, `PluginProvider` wraps `connectToHost()` in a component that
renders its children only once the handshake has completed, so consumers never have to
handle a not-yet-connected state themselves:

```tsx
import { PluginProvider, usePluginContext } from '@green-ecolution/plugin-interface'

function App() {
  return (
    <PluginProvider>
      <TreeImportStatus />
    </PluginProvider>
  )
}

function TreeImportStatus() {
  const { locale, theme, user, plugin } = usePluginContext()
  return (
    <p>
      {user.displayName} · {plugin.slug} · {locale} · {theme}
    </p>
  )
}
```

Two optional props cover the states before and instead of a completed handshake.
`pending` is rendered while it runs (nothing by default), and `fallback` receives the
error when the host never answered. Without a `fallback` the provider renders the error
message itself, so a plugin opened outside the host says so rather than showing a blank
page:

```tsx
<PluginProvider pending={<Spinner />} fallback={(error) => <ErrorPage error={error} />}>
  <TreeImportStatus />
</PluginProvider>
```

### `PluginContext`

```typescript
interface PluginContext {
  locale: 'de' | 'en'
  theme: 'light' | 'dark'
  user: { displayName: string }
  plugin: { slug: string }
}
```

This is presentation context only: a display name to greet the operator with, the
interface language and colour scheme to match, and the plugin's own slug. It carries no
token and no credential. A plugin's view runs as a visitor, not as an authenticated
API client; if your plugin needs to write data, that happens through your own backend
using the plugin's API key against the ingest endpoints below, never from the browser
using anything handed to it in `PluginContext`.

## The ingest contract

This part needs no SDK, just an HTTP client and the API key issued when the plugin was
installed. Send it as a bearer token:

```
Authorization: Bearer gep_<uuid>.<secret>
```

All ingest endpoints live under `/api/v1/plugins/ingest/` and one identity endpoint at
`/api/v1/plugins/me`; none of them require a user session. A disabled plugin, or an
instance with the plugins feature turned off, answers `403` / `503` respectively, with a
JSON body `{ "error": "...", "code": "..." }`.

### Confirm identity

```
GET /api/v1/plugins/me
```

```json
{
  "id": "01990000-0000-7000-8000-000000000001",
  "slug": "tbz-baumkataster",
  "name": "TBZ Baumkataster",
  "description": null,
  "organization_id": "01980000-0000-7000-8000-000000000001",
  "permissions": ["tree:create", "tree:update"],
  "required_permissions": ["tree:read"],
  "frontend_mode": "none",
  "frontend_target": null,
  "enabled": true,
  "has_credential": true,
  "last_seen_at": "2026-09-01T10:00:00Z",
  "created_at": "2026-08-01T00:00:00Z"
}
```

Use this before importing anything to confirm the plugin is enabled and to read its own
`organization_id` and granted `permissions` rather than hard-coding them.

### Upsert trees in a batch

```
POST /api/v1/plugins/ingest/trees
```

Up to 500 entries per request, matched by `external_id`, an opaque identifier your
adapter invents and reuses on every later run for the same tree. Requires `tree:create`
and `tree:update` among the plugin's own permissions.

```json
{
  "items": [
    {
      "external_id": "12345",
      "number": "FL-001",
      "species": "Quercus robur",
      "planting_year": 1998,
      "latitude": 54.7836,
      "longitude": 9.4321,
      "description": null,
      "additional_info": { "objectid": 12345 }
    }
  ]
}
```

Every entry is processed independently, so one malformed entry never fails the rest of
the batch; the response is `200` even when some entries failed:

```json
{
  "results": [
    {
      "external_id": "12345",
      "status": "created",
      "tree_id": "01990000-0000-7000-8000-000000000001"
    }
  ],
  "summary": { "created": 1, "updated": 0, "unchanged": 0, "failed": 0 }
}
```

`status` is one of `created`, `updated`, `unchanged` or `failed`. A `failed` entry
carries an `error` string instead of a `tree_id`. Running the same batch again once
nothing has actually changed reports `unchanged`, so a full re-import on every run is
safe and cheap to repeat. A request with more than 500 items is rejected outright with
`413` and code `plugin.batch_too_large`, before any entry is processed.

### Delete a tree by external id

```
DELETE /api/v1/plugins/ingest/trees/{external_id}
```

Deletes the tree this `external_id` resolves to and returns `204`. Requires
`tree:delete`. Answers `404` if this plugin has no reference under that `external_id`.

### List your own tree references

```
GET /api/v1/plugins/ingest/trees?limit=100&cursor=12344
```

A keyset page over this plugin's own `external_id → tree` mappings, ordered by
`external_id`, useful for reconciling what Green Ecolution still has on file against
your own source system:

```json
{
  "items": [{ "external_id": "12345", "tree_id": "01990000-0000-7000-8000-000000000001" }],
  "next_cursor": "12345"
}
```

Pass `next_cursor` back as `cursor` to fetch the next page; a response with no
`next_cursor` is the last page. There is no `total`; walk the pages to the end rather
than trying to estimate how many are left.

## What a plugin cannot do

A plugin acts only within the organization it was installed into and only with the
permissions it was granted there, exactly like a user with a role scoped to that
organization. It cannot see or modify data belonging to another organization, and it
cannot exceed the permission set the installing administrator gave it. A tree it
created can later be moved to another organization by an administrator; the plugin's
own reference to it survives that move, but further updates or deletes then require
the plugin to still hold the matching permission in the tree's _new_ organization.

## License

AGPL-3.0-only
