# Changelog

All notable changes to Green Ecolution will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Note on Versioning:** This project was reset to v0.1.0 starting from the consolidated
> monorepo structure. Previous versions (v1.0.0 - v1.2.1) are preserved as legacy releases
> from the early development phase before the backend and frontend were merged into a
> single repository. For legacy releases, see the
> [GitHub Releases](https://github.com/green-ecolution/green-ecolution/releases) page.

## [0.2.0](https://github.com/green-ecolution/green-ecolution/compare/v0.1.2...v0.2.0) (2026-06-01)


### Features

* add Rust backend domain layer with entities and DTOs ([9557653](https://github.com/green-ecolution/green-ecolution/commit/95576535f9041a9d5801fa41731347725f4385b1))
* **auth:** wire PKCE end-to-end for the public frontend client ([a6ca10c](https://github.com/green-ecolution/green-ecolution/commit/a6ca10c7de78e6d590d6d562698fff29e3083de2))
* **backend-rs:** add API versioning, region CRUD, and path extraction fix ([9392a0f](https://github.com/green-ecolution/green-ecolution/commit/9392a0f857cfada87165eadb6a28ab2cc26beda9))
* **backend-rs:** add Application startup and configuration ([fcfaf67](https://github.com/green-ecolution/green-ecolution/commit/fcfaf673e15b11bceca5323258d581dcff3922d4))
* **backend-rs:** add Axum HTTP server with Region vertical slice ([d862d0b](https://github.com/green-ecolution/green-ecolution/commit/d862d0b7b77f7279dfcdfb017b242b112163503f))
* **backend-rs:** add empty domain crate to workspace ([f46e2e3](https://github.com/green-ecolution/green-ecolution/commit/f46e2e3b282535e6490a5be662d607f27ec2f3cf))
* **backend-rs:** add EventBus::publish_all default method ([15bf9d4](https://github.com/green-ecolution/green-ecolution/commit/15bf9d43b902d1dba9ff9acf24ff2dda67e83081))
* **backend-rs:** add missing endpoint stubs for info and tree nearest ([fd8e0dd](https://github.com/green-ecolution/green-ecolution/commit/fd8e0ddbe365c161965b9333c8cf31e5e77d7cb6))
* **backend-rs:** add OpenAPI docs for user and plugin endpoints ([433084e](https://github.com/green-ecolution/green-ecolution/commit/433084e48834be04e425a2563f234d13904c2de1))
* **backend-rs:** add OpenAPI path annotations to all handlers ([9a3f5d7](https://github.com/green-ecolution/green-ecolution/commit/9a3f5d769262b946c6e4e6be6a9548bcbd1b886c))
* **backend-rs:** add pagination support for list endpoints ([f0d5e78](https://github.com/green-ecolution/green-ecolution/commit/f0d5e786f56329c7ef15a1ce9d5958034cdebfaa))
* **backend-rs:** add repository traits, DTOs, and idiomatic Rust improvements ([173cbe8](https://github.com/green-ecolution/green-ecolution/commit/173cbe8f6cae753bb3b756cc7fdda65ce26cb793))
* **backend-rs:** add request-id tracing and 5xx error logging ([2f67c4d](https://github.com/green-ecolution/green-ecolution/commit/2f67c4d3a18585f18fa87b4b8251614f329a71e9))
* **backend-rs:** add sensor models, abilities, and lorawan sub-table migration ([c662699](https://github.com/green-ecolution/green-ecolution/commit/c662699e83777785b75865140babfc9bdee61910))
* **backend-rs:** add service layer with event-driven side effects ([9feee5f](https://github.com/green-ecolution/green-ecolution/commit/9feee5fd8cc51bac79bef125157abda81438c647))
* **backend-rs:** add structured tracing and request logging ([da63817](https://github.com/green-ecolution/green-ecolution/commit/da638175f4b772f1a4c4435b03a06cdf5df7bd63))
* **backend-rs:** add TreeReader::by_sensor_id and by_cluster_id ([344b737](https://github.com/green-ecolution/green-ecolution/commit/344b737ba0375fbac94d61a065c40f68bd844321))
* **backend-rs:** add v1 HTTP layer with DTOs, handlers, and routing ([5d2ae08](https://github.com/green-ecolution/green-ecolution/commit/5d2ae087f1136087896b966e0e8ee18f27bef58c))
* **backend-rs:** add validation error and shared value-object foundations ([be2cb13](https://github.com/green-ecolution/green-ecolution/commit/be2cb134d7c59cc3f450d3e8b72daf6496068d9d))
* **backend-rs:** configure OpenAPI server URL per environment ([f1c38b9](https://github.com/green-ecolution/green-ecolution/commit/f1c38b99bfd14eecf53e87d13e0c23fca54d04be))
* **backend-rs:** extend DomainEvent vocabulary with fine-grained tree events ([4028f18](https://github.com/green-ecolution/green-ecolution/commit/4028f183af2bd19e4e418fb576fb48a903e6f961))
* **backend-rs:** implement all CRUD handlers with integration tests ([5981a12](https://github.com/green-ecolution/green-ecolution/commit/5981a12d73284ca0d234e9a5d133efff7ba9fa2d))
* **backend-rs:** implement evaluation handler with integration tests ([1cc0953](https://github.com/green-ecolution/green-ecolution/commit/1cc095368ae824e3bc338ea3b6e73d4e57a59357))
* **backend-rs:** implement info handler with SystemInfoProvider ([e6d53f9](https://github.com/green-ecolution/green-ecolution/commit/e6d53f95467e458d2ea9e047a4a8b301f214cd05))
* **backend-rs:** implement info sub-handlers (map, server, services, statistics) ([bc2d21d](https://github.com/green-ecolution/green-ecolution/commit/bc2d21ded0b44b07846d32db920b8dffd3efd192))
* **backend-rs:** instrument handlers, services and repositories with tracing ([a9dec08](https://github.com/green-ecolution/green-ecolution/commit/a9dec08395b7f9478eaa401cee5384f2e2558523))
* **backend-rs:** integrate utoipa for OpenAPI documentation ([f2960d6](https://github.com/green-ecolution/green-ecolution/commit/f2960d6b80de0d07778dbdeade5c2a3bbcf0c3c8))
* **backend-rs:** keycloak/oidc auth with jwt validation and demo bypass ([51f390b](https://github.com/green-ecolution/green-ecolution/commit/51f390b03439b1f3b4b0cafeed7ea8ebb2dd374a))
* **backend-rs:** move logging and pool config to YAML ([997c2de](https://github.com/green-ecolution/green-ecolution/commit/997c2de939a5802d95c3413f1691505a3ca7b800))
* **backend-rs:** MQTT sensor ingest with auto-link and watering subscriber ([0400825](https://github.com/green-ecolution/green-ecolution/commit/0400825f8f55d835d912d2cc4a64d34e09daa35e))
* **backend-rs:** wire CORS layer with config-driven origins ([5e09ffe](https://github.com/green-ecolution/green-ecolution/commit/5e09ffe352fda5d88d45bcec33d7ef8e107304dd))
* **backend-rs:** wire rust backend into dev stack with migrate binary ([68fa683](https://github.com/green-ecolution/green-ecolution/commit/68fa683a65aa310e8b7fbb01b2cace3747c03b9a))
* **backend:** add nearest-tree API endpoint (GECO-76) ([#750](https://github.com/green-ecolution/green-ecolution/issues/750)) ([0a251b6](https://github.com/green-ecolution/green-ecolution/commit/0a251b62aeb50ecf32f341cee964dd72b27b1746))
* disable routing and plugin features for release ([#794](https://github.com/green-ecolution/green-ecolution/issues/794)) ([730a348](https://github.com/green-ecolution/green-ecolution/commit/730a348bc31ec866050d13d55b404ffbbece2328))
* **domain-wasm:** add crate skeleton and register in workspace ([5b0a2d9](https://github.com/green-ecolution/green-ecolution/commit/5b0a2d95a582918f4e3dd989fadf7a7c567a7300))
* **domain-wasm:** add defaultMessages table for ValidationIssue keys ([a82c8d6](https://github.com/green-ecolution/green-ecolution/commit/a82c8d6b92b934fc9ca61f26f057a5802bc9c86e))
* **domain-wasm:** add frontend package config and shared types ([510ebf3](https://github.com/green-ecolution/green-ecolution/commit/510ebf3108db3b92667d699a0ecbc8b9b985b5ba))
* **domain-wasm:** add RHF resolvers for tree/cluster/vehicle/watering-plan drafts ([536decc](https://github.com/green-ecolution/green-ecolution/commit/536decc7425d6b649223700d64ec19346d15171f))
* **domain-wasm:** add tree draft aggregate validator ([f40fa9a](https://github.com/green-ecolution/green-ecolution/commit/f40fa9a48fc17e44f6f2f1c5cab599e0d0d6552e))
* **domain-wasm:** add tree-cluster draft validator ([e85dddb](https://github.com/green-ecolution/green-ecolution/commit/e85dddb62cee983ffd6a4a48f2ad97ac102a8e7e))
* **domain-wasm:** add vehicle draft validator ([ba10382](https://github.com/green-ecolution/green-ecolution/commit/ba10382248f6e347ba3a2f128526338570876566))
* **domain-wasm:** add watering-plan draft validator ([1c95262](https://github.com/green-ecolution/green-ecolution/commit/1c95262d005392edc1b0727c79418f9ccfa93626))
* **domain-wasm:** map ValidationError to ValidationIssue ([5461f28](https://github.com/green-ecolution/green-ecolution/commit/5461f283572a5f5413a37faafec95499c75afbc6))
* **domain-wasm:** per-value-object field validators ([31766cb](https://github.com/green-ecolution/green-ecolution/commit/31766cbb882355d9197079d8283e38d85fc2c2cf))
* **domain:** add BoundingBox value object ([ad2206e](https://github.com/green-ecolution/green-ecolution/commit/ad2206ef157f6312291cbd33ed0aa487a19043ef))
* **domain:** add ClusterMarker projection ([b3bfc97](https://github.com/green-ecolution/green-ecolution/commit/b3bfc97e2787365ae18a24b7e939941210f70003))
* **domain:** add optional bbox to TreeSearchQuery ([e5e37c9](https://github.com/green-ecolution/green-ecolution/commit/e5e37c9ff86276fb563d1962aef8b4dcf08a7fc6))
* **domain:** add sensor_model module with SensorModel + abilities ([324c090](https://github.com/green-ecolution/green-ecolution/commit/324c090a61cb93507f15142c846e30ecb0f32f19))
* **domain:** add TreeMarker projection ([8e320e6](https://github.com/green-ecolution/green-ecolution/commit/8e320e686902ddf8eae8ea0447331cbfce28dafb))
* **domain:** add view_markers to reader traits ([0945f0b](https://github.com/green-ecolution/green-ecolution/commit/0945f0be93a718a8baeffca845f700bc37b04b79))
* **frontend:** add copy-to-clipboard button for sensor ID in QR scan result ([#747](https://github.com/green-ecolution/green-ecolution/issues/747)) ([e8c11eb](https://github.com/green-ecolution/green-ecolution/commit/e8c11eb256b1c4f80af745af6cce60b7be473147))
* **frontend:** add PWA support with service worker, splash screen and offline handling ([#739](https://github.com/green-ecolution/green-ecolution/issues/739)) ([8e77659](https://github.com/green-ecolution/green-ecolution/commit/8e77659bc4397d8f051f2cfe4ec5b326f068f23d))
* **frontend:** add QR code scanner for sensor identification ([#741](https://github.com/green-ecolution/green-ecolution/issues/741)) ([a76bb82](https://github.com/green-ecolution/green-ecolution/commit/a76bb8221023d2ea505f20b8a1f38b99bb91d13c))
* **frontend:** add tree/cluster marker queries ([935bc89](https://github.com/green-ecolution/green-ecolution/commit/935bc8967b1b2e33b7b2493d2f506bb65ebcbfc2))
* **frontend:** add useViewportBBox hook ([6e5d32c](https://github.com/green-ecolution/green-ecolution/commit/6e5d32c92d807e9302103e2cddb22594162fd431))
* **frontend:** capture GPS location during sensor onboarding ([#742](https://github.com/green-ecolution/green-ecolution/issues/742)) ([ae35fca](https://github.com/green-ecolution/green-ecolution/commit/ae35fcad2455450ad48c4f1e8d44938453ab819e))
* **frontend:** enable React Compiler ([#658](https://github.com/green-ecolution/green-ecolution/issues/658)) ([a2741b8](https://github.com/green-ecolution/green-ecolution/commit/a2741b87232e90485c48d9e0e971e4039bd7dc97))
* **frontend:** redesign debug dashboard with structured card layout ([#748](https://github.com/green-ecolution/green-ecolution/issues/748)) ([093595a](https://github.com/green-ecolution/green-ecolution/commit/093595af7d6afb21d26b4e369b8905dea0ac0c25))
* **frontend:** regenerate backend-client for sensor-models endpoints + handle optional sensor coordinate ([eef3883](https://github.com/green-ecolution/green-ecolution/commit/eef38833a2b8502970b0b3ef2123d45df33551e1))
* **frontend:** regenerate backend-client from Rust backend OpenAPI spec ([dad9346](https://github.com/green-ecolution/green-ecolution/commit/dad93467d96b281bff79faaeca20e82e4bd93e07))
* **frontend:** show nearest trees after sensor GPS capture (GECO-77) ([#751](https://github.com/green-ecolution/green-ecolution/issues/751)) ([3426316](https://github.com/green-ecolution/green-ecolution/commit/34263162901a67b179bcbaee0785a6da24e4ee7c))
* **http:** add /health liveness endpoint outside /api/v1 ([5658a2a](https://github.com/green-ecolution/green-ecolution/commit/5658a2a5ddd21f7cef6d7875f73a34195ef34cbe))
* **http:** add GET /clusters/markers endpoint ([5eec019](https://github.com/green-ecolution/green-ecolution/commit/5eec0191e71c83370329cb112c0ed559f28a220d))
* **http:** add GET /trees/markers endpoint ([2bd7a2a](https://github.com/green-ecolution/green-ecolution/commit/2bd7a2a8843a35d5f6b04baa67ef186d3096dce0))
* **http:** add marker response DTOs ([0c5b1ec](https://github.com/green-ecolution/green-ecolution/commit/0c5b1ec18d6bdd6c07aa7837fab2fe42c4dac23c))
* **http:** sensor create/activate endpoints + sensor models list ([a0af36d](https://github.com/green-ecolution/green-ecolution/commit/a0af36dd5eb65d26db59c84b90d2a3ac904db72c))
* **info:** add system info page with service status and version check ([#638](https://github.com/green-ecolution/green-ecolution/issues/638)) ([64860f8](https://github.com/green-ecolution/green-ecolution/commit/64860f8d6ab36de9dfcd3394821c26997b4c855c)), closes [#69](https://github.com/green-ecolution/green-ecolution/issues/69)
* manual tree selection in sensor activation (GECO-78) ([#800](https://github.com/green-ecolution/green-ecolution/issues/800)) ([be95129](https://github.com/green-ecolution/green-ecolution/commit/be95129becc4b773fbcfcad5ab04203b02d8bca5))
* migrate to uuid v7 ids ([#791](https://github.com/green-ecolution/green-ecolution/issues/791)) ([4812b05](https://github.com/green-ecolution/green-ecolution/commit/4812b0548d89be94a00028559de8038dd31c2533))
* rebuild info endpoint with real data ([#784](https://github.com/green-ecolution/green-ecolution/issues/784)) ([340333f](https://github.com/green-ecolution/green-ecolution/commit/340333fdaa52e50e36f601d832a9aceebd3cb0ff))
* **sensor-wizard:** merge GPS step into tree selection (GECO-130) ([#803](https://github.com/green-ecolution/green-ecolution/issues/803)) ([eeeb120](https://github.com/green-ecolution/green-ecolution/commit/eeeb120e8fa36a272e441484ead811a965f20997))
* **sensor:** add guided assignment wizard with database verification (GECO-79, GECO-64) ([#801](https://github.com/green-ecolution/green-ecolution/issues/801)) ([1f3becc](https://github.com/green-ecolution/green-ecolution/commit/1f3becc969dd5305fff18fc4f434d3df1c38a390))
* **sensor:** redesign sensor detail page ([#785](https://github.com/green-ecolution/green-ecolution/issues/785)) ([657e318](https://github.com/green-ecolution/green-ecolution/commit/657e3185a33972b37d1c5f58dda24e151bc2fe18))
* **server:** expose view_markers on tree and cluster services ([cad0553](https://github.com/green-ecolution/green-ecolution/commit/cad055320d2d0bc1c3aa4058e092c8f6fd36cd6f))
* **server:** graceful config-load error handling ([7370034](https://github.com/green-ecolution/green-ecolution/commit/7370034231c04aa1f95de193ad1121c50b1fcb38))
* **server:** implement GET /v1/trees/nearest endpoint ([579053f](https://github.com/green-ecolution/green-ecolution/commit/579053f050a436e89d870b111ee555e6fee1268f))
* **server:** pg impls of view_markers ([0f18d61](https://github.com/green-ecolution/green-ecolution/commit/0f18d61f07f04395fac40badc57552b1de1c838c))
* **server:** sensor create/activate/ingest service + MQTT dispatch per model ([8975518](https://github.com/green-ecolution/green-ecolution/commit/89755184203de98950d8d65035169a6e54756260))
* **ui:** add date picker component for date input fields ([#673](https://github.com/green-ecolution/green-ecolution/issues/673)) ([1ce260b](https://github.com/green-ecolution/green-ecolution/commit/1ce260b04935711a304e4639dd013b5bc8a95777)), closes [#120](https://github.com/green-ecolution/green-ecolution/issues/120)


### Bug Fixes

* add driving license hierarchy validation and form state reliability ([#674](https://github.com/green-ecolution/green-ecolution/issues/674)) ([59b2629](https://github.com/green-ecolution/green-ecolution/commit/59b262990aeb7c3849650145796992a8d08686b0))
* **api:** derive OpenAPI version from Cargo.toml ([f7ed86c](https://github.com/green-ecolution/green-ecolution/commit/f7ed86c4d638787ca7204f7c29eeaa1f6142e911))
* **api:** drop duplicate /api segment from client URLs ([#792](https://github.com/green-ecolution/green-ecolution/issues/792)) ([0121a23](https://github.com/green-ecolution/green-ecolution/commit/0121a23ca316138620ba67c101acf4852a548079))
* **backend-rs:** add license info to OpenAPI spec for client generation ([878dd9f](https://github.com/green-ecolution/green-ecolution/commit/878dd9fdee65290b1618343b874a76864946b4e9))
* **backend-rs:** fix ListResponse schema, pagination naming, and cluster alias ([080c70f](https://github.com/green-ecolution/green-ecolution/commit/080c70f2ad7405e611d46ba841afc2fe75cb5c0b))
* **backend-rs:** update integration tests for pagination field rename ([bda0f79](https://github.com/green-ecolution/green-ecolution/commit/bda0f79fc2f42b2089726314762ad3e1e663c5b1))
* **backend:** adapt RSA test keygen to rand 0.10 / drop unused rand dep ([91cf430](https://github.com/green-ecolution/green-ecolution/commit/91cf430757d262bdb3fed487cc060ceb805c5e9f))
* **backend:** reject past dates in watering plan create and update ([#662](https://github.com/green-ecolution/green-ecolution/issues/662)) ([b1636eb](https://github.com/green-ecolution/green-ecolution/commit/b1636eba7d0bdb62d7dacf4ef12fa986447c5055)), closes [#642](https://github.com/green-ecolution/green-ecolution/issues/642)
* **ci:** drop --offline + use cargo update -p for Cargo.lock sync ([06a7a6f](https://github.com/green-ecolution/green-ecolution/commit/06a7a6ff871599662c4a8b56a21d353c1affe40c))
* **ci:** ignore wasm-pack output in prettier and pre-build it for Dockerfile test ([c7e3e4f](https://github.com/green-ecolution/green-ecolution/commit/c7e3e4fcb299f3dcb3909e73bbb7ecca1533711c))
* **ci:** shorten commit sha in stage version string ([#793](https://github.com/green-ecolution/green-ecolution/issues/793)) ([2262bad](https://github.com/green-ecolution/green-ecolution/commit/2262bad931cef49ef96d6238123172bdb3acf278))
* **domain-wasm:** add rlib crate-type and drop duplicate serde_json dev-dep ([4bcfc9c](https://github.com/green-ecolution/green-ecolution/commit/4bcfc9cfd956ca76d772f39721fc09d638110ba4))
* **domain-wasm:** expose resolvers as typed factories instead of as Resolver&lt;any&gt; ([0f20ecb](https://github.com/green-ecolution/green-ecolution/commit/0f20ecb790841eb900a339a59f4bc2045d9c3075))
* **domain-wasm:** parameterize resolver types, add TreeForm.provider, expose src directly ([75ab0ad](https://github.com/green-ecolution/green-ecolution/commit/75ab0adf5256682b4938db690d29fbd6a528eac4))
* **domain-wasm:** tolerate string number inputs and surface unparsable as ValidationIssue ([e1c8aa4](https://github.com/green-ecolution/green-ecolution/commit/e1c8aa4e62d142cd5ae3dc79959566e2200bf382))
* **domain-wasm:** use serde derives on domain enums + normalise Date inputs ([e257e89](https://github.com/green-ecolution/green-ecolution/commit/e257e893b8cd3fcd026028c7163870cc5f39a8f1))
* **domain-wasm:** validate vehicle type/status/driving_license fields ([2cb8114](https://github.com/green-ecolution/green-ecolution/commit/2cb8114057a089fbe9eacbcfaa7872c64f9b77f3))
* **frontend:** fix API base path and disable auth for Rust backend ([3142e46](https://github.com/green-ecolution/green-ecolution/commit/3142e46cf1c45d128b3ef4b385eb3544d121d0f9))
* **frontend:** fix save button disabled on finished watering plan ([#663](https://github.com/green-ecolution/green-ecolution/issues/663)) ([ae9b892](https://github.com/green-ecolution/green-ecolution/commit/ae9b892f69edf1344bd90f35529670e34ca10b75)), closes [#641](https://github.com/green-ecolution/green-ecolution/issues/641)
* **frontend:** move bboxRef write into useEffect to satisfy react-hooks/refs ([a116e23](https://github.com/green-ecolution/green-ecolution/commit/a116e23ea01284c2f53324da354108d447ea8a5d))
* **http:** make watering_status filter accept query strings ([4ca2ab8](https://github.com/green-ecolution/green-ecolution/commit/4ca2ab8a8f8f1d2e2fd83161039dfdf0bf647358))
* **k8s:** align stage/demo/prod configs backend env vars ([2262bad](https://github.com/green-ecolution/green-ecolution/commit/2262bad931cef49ef96d6238123172bdb3acf278))
* **k8s:** disable postgres TLS requirement in stage/demo/prod ([2262bad](https://github.com/green-ecolution/green-ecolution/commit/2262bad931cef49ef96d6238123172bdb3acf278))
* **mqtt:** handle real TTN uplinks for GES-1000 ([#802](https://github.com/green-ecolution/green-ecolution/issues/802)) ([fc91dac](https://github.com/green-ecolution/green-ecolution/commit/fc91dacef2c23051c9d1c91fabb6ffb2da2945b1))
* **seeds:** rebuild geometry from lat/lng in proper PostGIS order ([9b16e82](https://github.com/green-ecolution/green-ecolution/commit/9b16e8265619fa9661f3023d2f5f95ae699b1677))


### Performance Improvements

* **frontend:** reduce unnecessary data fetching and fix navbar hover stutter from event bubbling ([#749](https://github.com/green-ecolution/green-ecolution/issues/749)) ([1389ba0](https://github.com/green-ecolution/green-ecolution/commit/1389ba02c23db4ed1ef2e5dbdc0bbc63c2e5d62d))

## [0.1.2](https://github.com/green-ecolution/green-ecolution/compare/v0.1.1...v0.1.2) (2026-02-03)


### Features

* **auth:** allow unauthorized requests to pass through jwt middleware if configured ([#634](https://github.com/green-ecolution/green-ecolution/issues/634)) ([24f61c3](https://github.com/green-ecolution/green-ecolution/commit/24f61c399f4d5d6df695544853705aa5d6383867))
* **auth:** fetch OIDC public key dynamically from JWKS endpoint ([#595](https://github.com/green-ecolution/green-ecolution/issues/595)) ([c43bfc4](https://github.com/green-ecolution/green-ecolution/commit/c43bfc43b1b3b89434701e906d0d00f02783831e))
* **config:** make config file optional, use env vars as primary source ([#594](https://github.com/green-ecolution/green-ecolution/issues/594)) ([75e0851](https://github.com/green-ecolution/green-ecolution/commit/75e0851296430ca6b58d832d62d6bbd513af7b19))
* **filter:** add slider component for planting year filter with dynamic years ([#636](https://github.com/green-ecolution/green-ecolution/issues/636)) ([4b22c79](https://github.com/green-ecolution/green-ecolution/commit/4b22c79e3098792ce73140e2e02b836327eae2a6))
* **ui:** add @green-ecolution/ui package with shared components and Storybook ([#591](https://github.com/green-ecolution/green-ecolution/issues/591)) ([443ce11](https://github.com/green-ecolution/green-ecolution/commit/443ce115d668ea5d495d48a5504f9a46985a04b1))
* **ui:** add filter functionality to map CRUD selection pages ([#637](https://github.com/green-ecolution/green-ecolution/issues/637)) ([c8c5702](https://github.com/green-ecolution/green-ecolution/commit/c8c57028235b22335e3aee5508beb2317bffca1a)), closes [#146](https://github.com/green-ecolution/green-ecolution/issues/146)
* **ui:** migrate components to shared UI package and Tailwind v4 compatibility ([#608](https://github.com/green-ecolution/green-ecolution/issues/608)) ([a78695e](https://github.com/green-ecolution/green-ecolution/commit/a78695e187af770cc1afb3b261fa9c28cc45cff4))


### Bug Fixes

* **frontend:** improve auth token handling with proactive refresh ([#618](https://github.com/green-ecolution/green-ecolution/issues/618)) ([3b74d5a](https://github.com/green-ecolution/green-ecolution/commit/3b74d5afef223cd88470b8fa26675f9ef6e8ec89)), closes [#607](https://github.com/green-ecolution/green-ecolution/issues/607)
* **frontend:** resolve map filter for cluster membership ([#628](https://github.com/green-ecolution/green-ecolution/issues/628)) ([5d860bc](https://github.com/green-ecolution/green-ecolution/commit/5d860bc19905d1d1415545b2369b384a47277f74))
* **frontend:** resolve NaN entity ID on page refresh ([#619](https://github.com/green-ecolution/green-ecolution/issues/619)) ([ec7172b](https://github.com/green-ecolution/green-ecolution/commit/ec7172bdbbd6959ef9d26f9ed9f367a9f5f49a17))
* **seed:** overhaul vehicle seed data with realistic values ([#635](https://github.com/green-ecolution/green-ecolution/issues/635)) ([24a67b6](https://github.com/green-ecolution/green-ecolution/commit/24a67b6583f0e15cc629515e296d3def5e620838)), closes [#630](https://github.com/green-ecolution/green-ecolution/issues/630) [#631](https://github.com/green-ecolution/green-ecolution/issues/631)

## [0.1.1](https://github.com/green-ecolution/green-ecolution/compare/v0.1.0...v0.1.1) (2026-01-11)


### Features

* **ci:** add PR status tracking to GitHub Project ([b68b011](https://github.com/green-ecolution/green-ecolution/commit/b68b011f04c843f3d4342a24d4dfdf515a081153))
* **frontend:** update footer links and add version display ([#586](https://github.com/green-ecolution/green-ecolution/issues/586)) ([adda771](https://github.com/green-ecolution/green-ecolution/commit/adda77152bc6cbd0d396e4c19b18c5ceb7e8aeb2)), closes [#585](https://github.com/green-ecolution/green-ecolution/issues/585)


### Bug Fixes

* **backend:** save description when creating a tree ([#578](https://github.com/green-ecolution/green-ecolution/issues/578)) ([fffbb79](https://github.com/green-ecolution/green-ecolution/commit/fffbb797ee2e66f6c0859e47b730eeb9d7b19a33)), closes [#570](https://github.com/green-ecolution/green-ecolution/issues/570)
* **deploy:** update staging OIDC configuration for Keycloak ([#567](https://github.com/green-ecolution/green-ecolution/issues/567)) ([5804a37](https://github.com/green-ecolution/green-ecolution/commit/5804a37fdac12104143dd8a0472a05048758716b)), closes [#566](https://github.com/green-ecolution/green-ecolution/issues/566)
* **frontend:** convert treeClusterId to number on form submit ([#514](https://github.com/green-ecolution/green-ecolution/issues/514)) ([05890d4](https://github.com/green-ecolution/green-ecolution/commit/05890d413982e847f4a17690da1fae070d926ddc))
* **frontend:** correct form navigation blocker logic to only block unsaved changes ([#517](https://github.com/green-ecolution/green-ecolution/issues/517)) ([04b3114](https://github.com/green-ecolution/green-ecolution/commit/04b311418b9658ed7aa210fd0a823f9d23b93818))
* **frontend:** improve Select component click handling and accessibility ([#524](https://github.com/green-ecolution/green-ecolution/issues/524)) ([7b929be](https://github.com/green-ecolution/green-ecolution/commit/7b929bee1458a04c8d826aa41ac8970400a51319))
* **frontend:** prevent duplicate treecluster selection on rapid clicks ([#580](https://github.com/green-ecolution/green-ecolution/issues/580)) ([9162330](https://github.com/green-ecolution/green-ecolution/commit/916233077e2ea27fc3e6c1803617850001ac7704)), closes [#85](https://github.com/green-ecolution/green-ecolution/issues/85)
* **frontend:** prevent route preloading for login and logout links ([#531](https://github.com/green-ecolution/green-ecolution/issues/531)) ([97f9a23](https://github.com/green-ecolution/green-ecolution/commit/97f9a23d4ceb4bfb7a3546467bd8f070bdfa039f))
* **frontend:** prevent search engine indexing ([#532](https://github.com/green-ecolution/green-ecolution/issues/532)) ([bcd0c70](https://github.com/green-ecolution/green-ecolution/commit/bcd0c706525339bd964db98bdbc09efefb4a0dd7))
* remove component prefix from release tags ([85c2744](https://github.com/green-ecolution/green-ecolution/commit/85c2744cf8fc95a45e701fa7ebdada7165269dcf))


### Performance Improvements

* **frontend:** optimize matchMedia usage with reactive hook ([#533](https://github.com/green-ecolution/green-ecolution/issues/533)) ([258e88c](https://github.com/green-ecolution/green-ecolution/commit/258e88c706dde6060e9e922dc762bc4478eb9521))
* **map:** optimize marker rendering and map interaction performance ([#561](https://github.com/green-ecolution/green-ecolution/issues/561)) ([a653f85](https://github.com/green-ecolution/green-ecolution/commit/a653f85278442e2cc765839cd160ad10b07f9846))

## 0.1.0 (2025-12-29)

### Features

* activate tsc -b on build ([df25477](https://github.com/green-ecolution/green-ecolution/commit/df25477c9ac147f981ca12d9e49b55a8b5834cda))
* add CODEOWNERS ([e66b4ff](https://github.com/green-ecolution/green-ecolution/commit/e66b4ff5b87375e2c9f82425833157b068e7a98f))
* add frontend in compose.app.yaml ([bc21f78](https://github.com/green-ecolution/green-ecolution/commit/bc21f78d06347d4789427c6277bb8500e02051ef))
* change vite config to localhost in proxy route /api-local ([0b1ade5](https://github.com/green-ecolution/green-ecolution/commit/0b1ade57ae4e6e0541d3e95d5eb717bd0ca2f684))
* check in backend api client src ([e9d82ab](https://github.com/green-ecolution/green-ecolution/commit/e9d82abdfbb1781f687b781e8fd70970db8f4575))
* checkin go generated code ([eaf35d3](https://github.com/green-ecolution/green-ecolution/commit/eaf35d314a2f52cce77ce49477c14e3a80761f69))
* direnv use flake only when nix is installed ([ab6d7a1](https://github.com/green-ecolution/green-ecolution/commit/ab6d7a18637e693910d9aa3e01e29640bd2b8649))
* disable eslint rule void return in promises ([9ca4feb](https://github.com/green-ecolution/green-ecolution/commit/9ca4feb90c01883d637f739453ecee2e52c6c5e5))
* embed frontend in binary ([ac835cf](https://github.com/green-ecolution/green-ecolution/commit/ac835cf515a261fb55d6272a5cbc26b957f025ca))
* improve fl seed data ([05b2783](https://github.com/green-ecolution/green-ecolution/commit/05b2783bcb9f7750e5f1d6acfc08c3edaefcb3ed))
* match keycloak user id in realm export with user in seed data ([1917fd7](https://github.com/green-ecolution/green-ecolution/commit/1917fd7fbc62b073b4aded34b9f42065b70671f4))
* merge flake.nix from backend and frontend and build package with nix ([2756a3e](https://github.com/green-ecolution/green-ecolution/commit/2756a3e19013c7f36ab9156aac18cf32aa2f7b21))
* merge flake.nix from backend and frontend and build package with nix ([2756a3e](https://github.com/green-ecolution/green-ecolution/commit/2756a3e19013c7f36ab9156aac18cf32aa2f7b21))
* migrate backend makefile to root and add frontend scripts ([4674c4d](https://github.com/green-ecolution/green-ecolution/commit/4674c4d117984c784639e9bdad942478015f82c6))
* override postgres date types to time.Time ([787b2b9](https://github.com/green-ecolution/green-ecolution/commit/787b2b906c2ae4a441aad258ed1e3348ce20bd8b))
* reduce Dockerfile to only one and add compose.app.yaml to run backend through docker compose ([9134b9b](https://github.com/green-ecolution/green-ecolution/commit/9134b9b9d4fd20f903e2d7f25205af2059a2732e))
* remove unused zustand form store ([7b6758a](https://github.com/green-ecolution/green-ecolution/commit/7b6758acf3f711c17a91d30a1c6ef229ce637d03))
* run infra in compose.yaml at root dir ([0554849](https://github.com/green-ecolution/green-ecolution/commit/05548492ae01d1f139a6b93aa1781196490e9b27))
* set fix version in dockerfile for backend api generator ([d4dbf41](https://github.com/green-ecolution/green-ecolution/commit/d4dbf41dcaf66667fd72f0147ca1f933e98f1f06))
* update .dockerignore ([617b112](https://github.com/green-ecolution/green-ecolution/commit/617b112646766d9143515964913a24121af617be))
* update flake.nix to monorepo and generate dev vm ([ca66ad3](https://github.com/green-ecolution/green-ecolution/commit/ca66ad314b511d807c5db075ae904a2fb0459949))
* update nix flake and add update script ([9041a71](https://github.com/green-ecolution/green-ecolution/commit/9041a7106ec5c08db8ed5001fa97b58160c64d9a))
* update tree cluster formular (create and update) ([d7de41a](https://github.com/green-ecolution/green-ecolution/commit/d7de41af117e469968f987ee8ae36eaa48ce21c4))
* update tree form schema (create and update) ([72b9668](https://github.com/green-ecolution/green-ecolution/commit/72b9668bdc1efa6fa40340990dabf6af6d0d011e))
* update vehicle schema and formular (create and update) ([c5901cf](https://github.com/green-ecolution/green-ecolution/commit/c5901cf13cb291715b05870202939dcba08bd499))
* update wateringplan schema and formular (create, update and update status) ([a027b9d](https://github.com/green-ecolution/green-ecolution/commit/a027b9dec2e40eef46aaf3c19afa2bae0020c883))

### Bug Fixes

* backend docker run ([b8896bd](https://github.com/green-ecolution/green-ecolution/commit/b8896bd47ebe6c6bbb2c0c0313b322ee49a3ba50))
* deploy demo postgres migration docker tag ([7baf810](https://github.com/green-ecolution/green-ecolution/commit/7baf81084442a0e30860fc09e8e59416fdefbba9))
* errors in codebase ([9963740](https://github.com/green-ecolution/green-ecolution/commit/99637409db597688644b6fabca695fce1c469054))
* generate openapi generator go backend client repo id ([243a3e2](https://github.com/green-ecolution/green-ecolution/commit/243a3e24896a22d590a69b4c310f69c5b584cffa))
* keycloak realm import ([3b27635](https://github.com/green-ecolution/green-ecolution/commit/3b2763580f326476e6e960dec29ccc889d1a5f0e))
* local docker compose ([39e6376](https://github.com/green-ecolution/green-ecolution/commit/39e6376d4432b4b4b93cacc05609c28917dd0979))
* only use flake when nix is installed ([a5cfb26](https://github.com/green-ecolution/green-ecolution/commit/a5cfb26d1586fcd1abe892b875b0573c0a0a4f08))
* pgadmin server json file ([c0a3911](https://github.com/green-ecolution/green-ecolution/commit/c0a3911f911d4ad02a69b48b4f516ef79e4f6547))
* pr pipelines ([485cb8f](https://github.com/green-ecolution/green-ecolution/commit/485cb8f8084deeaad99e5d643138feccd00006a0))
* stage deployment missing env ([e4cf8f4](https://github.com/green-ecolution/green-ecolution/commit/e4cf8f4efd2dfc8480ff2ba258eaef6983a55d34))
* use static release-please PR header ([feb906e](https://github.com/green-ecolution/green-ecolution/commit/feb906ee248ed6a8b7416e92b3fed06d99dcece5))
* vehicle type in vroom repository ([a374d8c](https://github.com/green-ecolution/green-ecolution/commit/a374d8cc3c7205a6d75328bd8d6e38a60771ee83))
