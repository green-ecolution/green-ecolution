# Urban data platforms: CIVITAS/CORE and CIVORA

How Green Ecolution can reach the two urban data platforms German municipalities use most,
for the data export, a future import, and dashboard embedding. Everything marked *verified*
was read in the platforms' public source code on 2026-11-09. Claims taken from studies or
marketing pages are marked as such. Re-check the sources before relying on this after a
platform release.

## Summary

| | CIVITAS/CORE V1 | CIVITAS/CORE V2 (Oct 2026) | CIVORA |
| --- | --- | --- | --- |
| Interface | OGC SensorThings API (FROST) | OGC SensorThings API (FROST), one endpoint per dataset | UrbanPulse REST API (no SensorThings) |
| External read | yes | yes | yes |
| External write | yes, separate write routes | **no**, gateway allows `GET` only | yes, `POST /Inbound` |
| Machine auth (default setup) | OAuth2 password grant, confidential client `api-access` with secret, scope `api:read` | OAuth2 password grant, public client `api-access`, no refresh token | HTTP Basic or Keycloak bearer, credentials issued by the operator |
| Anonymous read | public projects under `/context/sensorthings/public/` | datasets with open data access | no |
| Conventions for quantity, unit, depth | none | none (only `properties.reference`) | none, encoded in field names by convention |

Consequences:

- **Export.** Today's `SensorThingsSink` writes with `POST`/`PATCH` and authenticates with
  none, Basic or OAuth2 client credentials. That fits standalone FROST servers. It does not fit
  CIVITAS/CORE V2, which accepts no writes from outside. It does not fit CIVITAS/CORE V1 in its
  default setup either, because V1 has no client with a service account, so the export would
  need the password grant first. CIVORA needs its own UrbanPulse sink.
- **Data into CIVITAS/CORE V2** only arrives through the platform's own NiFi pipelines, whose
  sources are MQTT, SQL or a SensorThings API the pipeline polls. To serve V2, Green Ecolution
  would have to offer a readable SensorThings API (or MQTT) that a city's pipeline pulls from.
- **Import.** Both platforms leave quantity, unit and depth to whoever set up the data. An
  import therefore needs a per-series mapping (quantity, unit, depth), configured per source.
  It cannot rely on names or ObservedProperty definitions.
- **Do not claim** that CIVORA offers the SensorThings API or NGSI-LD. The BBSR study (2025)
  and Fraunhofer IESE say so, but none of DKSR's public repositories contains either.

## CIVITAS/CORE

Open source under EUPL-1.2 by Civitas Connect e.V. Sources:
`gitlab.com/civitas-connect/civitas-core/civitas-core-v2/{civitas-core-platform,civitas-core-deployment}`
and `gitlab.com/civitas-connect/civitas-core/civitas-core-v1/civitas-core`. In-repo docs live
in `gitlab.com/civitas-connect/civitas-core/documentation` (`docs_v2/` for V2, `docs/` for
V1). V1 is maintained until Q1 2027.

### V2: read path (verified)

- There is no instance-wide FROST endpoint. Every published dataset gets its own route,
  `https://api.<domain>/v1/datasets/<dataset-id>/<api-slug>/…`. APISIX rewrites it to
  `/FROST-Server/v1.1/Projects(<n>)/…`, so one dataset is one FROST project
  (`config-adapter-apisix/.../RouteAuthConfigurer.java`, `RouteUpstreams.java`). The URL is
  shown in the portal and returned by `GET https://management.<domain>/v1/datasets/<id>/apis`
  (`previewUrl`).
- Navigation links are rewritten onto the public route, so `@iot.nextLink` can be followed.
- FROST 2.8.1, `defaultTop` 100, `maxTop` 1000, `defaultCount` false (send `$count=true`
  explicitly). `$expand`, `$filter`, `$orderby` and `dataArray` are not restricted. There is no
  rate limit on the payload routes.
- FROST's MQTT interface is disabled (`mqtt.enabled: false`).
- A dataset in `DRAFT` or `READY` has no route and answers 404. Missing credentials answer
  401, missing permission 403.

### V2: no external writes (verified)

The OPA policy maps only `GET` for the FROST backend
(`authz/rego/data/backends/frost_server/data.json`: `"GET": "DATASET_PAYLOAD_READ"`). Any
other method has no permission entry and falls under the default deny. The V2 docs mention a
payload write permission, but no mapping for it exists in the code. Data enters V2 through
pipelines (NiFi) with three storage ports: `Things` (master data), `Observations`
(measurements into existing datastreams) and `ThingTree` (everything at once).

### V2: authentication (verified)

- Token: `POST https://idm.<domain>/realms/<instance-slug>/protocol/openid-connect/token`
  with `grant_type=password&client_id=api-access&username=…&password=…`.
- `api-access` is a public client with direct grants, without a service account and without
  refresh tokens (`components/frost/keycloak-clients.yaml`). The docs
  (`docs_v2/API/Authentication.md`) state that client credentials are not supported yet and
  announce more methods for the next release.
- The token must carry the audience `apisix-validator`. `api-access` adds it; tokens from the
  portal client are rejected.
- Permissions come from the platform's AuthZ repository, not from token roles. The caller
  needs `DATASET_PAYLOAD_READ` on the dataset, granted through a group with a role scoped to the
  dataset or its data pool. `DATASET_READ` (metadata only) is not enough.
- Datasets with `openDataAccess=true` can be read anonymously.
- There is no registration for third-party applications. A tenant admin creates a technical
  user and adds it to a group with a data consumer role.

### V2: data model (verified)

Data pool → datasets → data structures, pipelines and APIs. The only convention for
SensorThings entities is the upsert key `properties.reference` on Thing, Datastream and
Location. There is no vocabulary for ObservedProperties, no unit convention and no depth
property. No template for soil moisture exists; the model-forge schemas cover air quality,
smart meters and a grid use case.

### V1 differences (verified)

- Endpoint `https://api.<domain>/context/sensorthings/v1.1`, with separate read, write and
  delete routes. Unauthenticated reads of public projects under
  `/context/sensorthings/public/…`.
- FROST runs the Keycloak auth provider with project roles: global `read`, `create`, `update`,
  `delete`, `admin`, or per project `<project>__read` etc.
- `api-access` is confidential (client secret) with direct grants, without a service account.
  The gateway requires the scope `api:read`.

## CIVORA

Urban data platform of DKSR GmbH, sold as managed open source. Public sources:

- `gitlab.opencode.de/civora/civora-ui` (Vue, AGPL-3.0-only, CIVORA is a trademark).
- `gitlab.opencode.de/dksr-gmbh/DKSR-Open-UrbanPulse` (Java/Vert.x, last code change 2022,
  "German Free Software License", no LICENSE file). The deployed version is newer than this
  code.
- `gitlab.opencode.de/civora/fachanwendungen/hoferland-digital/airflow-pipelines-winterdienst`
  (AGPL-3.0). It vendors DKSR's own Python client `OUPClient` 3.5.0 under
  `third_party/civora_analytics_utils`. This is the best reference for how a deployed CIVORA
  instance behaves.
- The installation automation (Helm, Keycloak, backend configuration) and the connector SDK
  sources are not public.

### Interface: UrbanPulse (verified)

Each tenant has its own UrbanPulse host, for example `https://oup.hoferland.civora.org/`. It is
reachable from the internet and answers 401 with `WWW-Authenticate: Basic realm="UrbanPulse"`.

Read:

- `GET /UrbanPulseData/historic/sensordata` with exactly one of `SID`, `eventtype` (UUID) or
  `eventtypeName`, and optionally `since` and `until` (`yyyy-MM-dd'T'HH:mm:ss.SSSXX`; both or
  neither). Without a time range only the latest event per sensor is returned. `includeOnly=a,b`
  limits the fields.
- The response is a chunked JSON stream `{"sensordata":[{…}]}` or CSV (`Accept: text/csv`).
  There is no paging. A role may cap the range (`maxIntervalSizeInMinutes`). DKSR's own
  pipeline fetches seven-day windows.
- Auth is Basic or a Keycloak bearer token (`UpShiroAuth.java`). Read permissions are
  granted per SID and per event type. There is no self-service and no client credentials flow;
  the operator issues the credentials.
- Streaming via WebSocket `wss://<host>/OutboundInterfaces/outbound/<statement>` requires an
  admin to create an Esper statement first. Whether CIVORA exposes it is unknown.
- Discovery: the CIVORA data catalog (piveau, DCAT-AP) lists UrbanPulse URLs as distribution
  access URLs.

Write:

- Deployed CIVORA (from DKSR's client): `POST https://<host>/Inbound` with
  `Authorization: Basic …`, header `UrbanPulse-Timestamp` and the body
  `{"data":[{"SID":"…","timestamp":"…", …}]}`.
- The public 2022 code instead authenticates with
  `Authorization: UPConnector base64(connectorId):base64(HMAC-SHA256(connectorKey, timestamp + body))`,
  with the timestamp within ±15 minutes of server time. Success is 204.
- Before pushing, register in this order: event type schema (`POST /UrbanPulseManagement/api/schemas`,
  a JSON Schema with `title`, `properties`, `required`, `primaryKey`), connector
  (`POST …/api/connectors`), sensor (`POST …/api/sensors` with `senderid`, `eventtype`,
  `categories`, `description.reference`). The returned sensor id is the `SID`.

### Data model (verified)

An event type is a flat schema of named fields; `SID` and `timestamp` are added
automatically. A sensor (`SID`, a UUID) belongs to one event type and one connector and has
free-form `description` and `location`. Units and depths have no field of their own; DKSR puts
them into the field name (`soil_temperature_10cm_celsius`, `air_temperature_celsius`). Existing
event types cover weather (DWD, OpenWeatherMap, road weather); there is no soil moisture
example.

Unlike SensorThings, one UrbanPulse event carries all values of a device. That matches Green
Ecolution's own reading model (one uplink with all depths) better than one datastream per
depth.

### UI embedding (verified)

- The CIVORA UI has three iframe slots: `customIframeModule`, `customIframeModule2` and
  `customIframeModule3` (`civora-ui/config/config.js`). The operator configures them at
  container start through `VITE_CUSTOMER_MODULES_CUSTOM_IFRAME_MODULE[_2|_3]_*` (`URL`,
  `AUTHENTICATED_URL`, `REQUIRES_LOGIN`, `IS_PLUGIN_ENABLED`, `NAME`, `ICON`, `ROUTE`, `ORDER`
  and others). A tenant cannot add one through the UI.
- The iframe has no sandbox attribute. Once the user is logged in, `AUTHENTICATED_URL`
  replaces `URL`.
- Handshake (`src/composables/usePluginIntegration.ts`): the embedded page posts
  `{type: "GET_CIVORA_CONTEXT"}`. The host answers only if the sender's origin equals the
  iframe's origin, with
  `{type: "CIVORA_CONTEXT", payload: {user: {id, token}}, tenant: {id}, locale}`. `token` is the
  user's Keycloak access token. The host re-sends the context on every token refresh.

## Public SensorThings servers with soil moisture data

Useful for tests and for checking assumptions about real data. Neither is a CIVITAS/CORE or
CIVORA instance, and neither states a license.

- **Osnabrück**, `https://daten-api.osnabrueck.de/v1.1` (FROST). Three active ioplant-Tree
  sensors at young trees, soil moisture and pF at 30, 60 and 90 cm, one value every four hours.
  Shared ObservedProperty "Bodenfeuchte" (GEMET 7874), unit `ucum:%`. The depth is only free
  text in the datastream name ("Tiefe:30cm", "Tiefe: 90cm"). `observationType` is wrongly
  `OM_CategoryObservation`. Precipitation datastreams (`mm/h`) are on the same server.
- **Jena**, `https://udp-broker.nomad-dmz.jena.de/udp/rest/v1.1` (Eclipse sensiNact with a
  SensorThings layer). Over 60 soil sensors (TEROS 21 water potential, Dragino LSE01 VWC). Units
  are null, no depth is given, one ObservedProperty per datastream named after the resource,
  device metadata exposed as datastreams. `substringof()` fails; avoid `$select` with
  `$orderby`. Precipitation from weather stations is on the same server.

Neither server links a sensor to a tree.
