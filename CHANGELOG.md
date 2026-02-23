# Changelog

All notable changes to Green Ecolution will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Note on Versioning:** This project was reset to v0.1.0 starting from the consolidated
> monorepo structure. Previous versions (v1.0.0 - v1.2.1) are preserved as legacy releases
> from the early development phase before the backend and frontend were merged into a
> single repository. For legacy releases, see the
> [GitHub Releases](https://github.com/green-ecolution/green-ecolution/releases) page.

## [0.1.3](https://github.com/green-ecolution/green-ecolution/compare/v0.1.2...v0.1.3) (2026-02-23)


### Features

* **frontend:** enable React Compiler ([#658](https://github.com/green-ecolution/green-ecolution/issues/658)) ([a2741b8](https://github.com/green-ecolution/green-ecolution/commit/a2741b87232e90485c48d9e0e971e4039bd7dc97))
* **info:** add system info page with service status and version check ([#638](https://github.com/green-ecolution/green-ecolution/issues/638)) ([64860f8](https://github.com/green-ecolution/green-ecolution/commit/64860f8d6ab36de9dfcd3394821c26997b4c855c)), closes [#69](https://github.com/green-ecolution/green-ecolution/issues/69)
* **ui:** add date picker component for date input fields ([#673](https://github.com/green-ecolution/green-ecolution/issues/673)) ([1ce260b](https://github.com/green-ecolution/green-ecolution/commit/1ce260b04935711a304e4639dd013b5bc8a95777)), closes [#120](https://github.com/green-ecolution/green-ecolution/issues/120)


### Bug Fixes

* **backend:** reject past dates in watering plan create and update ([#662](https://github.com/green-ecolution/green-ecolution/issues/662)) ([b1636eb](https://github.com/green-ecolution/green-ecolution/commit/b1636eba7d0bdb62d7dacf4ef12fa986447c5055)), closes [#642](https://github.com/green-ecolution/green-ecolution/issues/642)
* **frontend:** fix save button disabled on finished watering plan ([#663](https://github.com/green-ecolution/green-ecolution/issues/663)) ([ae9b892](https://github.com/green-ecolution/green-ecolution/commit/ae9b892f69edf1344bd90f35529670e34ca10b75)), closes [#641](https://github.com/green-ecolution/green-ecolution/issues/641)

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
