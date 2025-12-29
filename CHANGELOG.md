# Changelog

All notable changes to Green Ecolution will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

## [Unreleased]

### Note on Versioning

This project was reset to v0.1.0 starting from the consolidated monorepo structure.
Previous versions (v1.0.0 - v1.2.1) are preserved as legacy releases from the early
development phase before the backend and frontend were merged into a single repository.

For legacy releases, see the [GitHub Releases](https://github.com/green-ecolution/green-ecolution/releases) page.
