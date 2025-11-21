# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [v1.2.1] - 2025-05-01

### Added

-   Configure Dependabot (#486)
-   Issue templates (#484)
-   CI/CD: release pipeline

### Changed

-   CI/CD: only run workflows when necessary (#485)
-   Rename repository from `github.com/green-ecolution/green-ecolution-backend` to `github.com/green-ecolution/green-ecolution/backend`
-   Remove dev deploy environment (`stage` will be used to deploy the develop branch, while `dev` is used to run it locally)
-   Dependency updates from dependabot
-   Update README.md

## [v1.2.0] - 2025-04-03

### Added

-   Telemetry (#90)
-   Pagination for treecluster, tree, sensor, region, watering plans and vehicles (#128)
-   Create a generic infrastructure for schedulars (#365)
-   Add provider and additional information fields to entities (#395)
-   Add Endpoint to unregister plugins (#402)
-   Extend plugin package and add refresh token function (#408)
-   Archive vehicles (#433)
-   Evaluation Service (#448)
-   Map sensor to tree when receiving sensor data (#452)
-   Implement filtering (#84)

### Changed

-   Add map boundings to config (#405)
-   Update go version to 1.24 (#420)
-   Change TTN attributes names (#440)
-   Add ability to disable services like `auth`, `routing`, `s3` and `mqtt` (#458)

### Fixed

-   Set watering status on sensor change (#409)
-   App panics if tree cluster has no region (#422)
-   Fix conn busy error message (#436)
-   Only one user needs required driving licences (#470)
-   Watering status is not updated on tree cluster create (#473)

### Removed

-   Flowerbeds (#407)
-   Images (#443)
-   Removed readonly attributes from tree entities (#428)
-   Remove DriverLicenseUnkown (#438)

## [v1.1.0] - 2024-01-13

### Added

-   Support for watering plans: storage, server layer, and additional properties (#278, #251).
-   Tracking water usage per tree cluster (#290).
-   Enhanced vehicle management with new fields and filtering capabilities (#243, #288).
-   Sensor data mapping logic and scheduler integration (#297, #322).
-   Map sensor to tree based on it's gps location (#259)
-   User management enhancements, including roles, statuses, and external authentication integration (#297).
-   Transaction in database operations (#235)
-   Event system to respond to specific events (#295)
-   Plugin system to integrate external plugins into the backend (#196)
-   Packages to share code, register plugin and communicate to the backend REST API (#271, #250)
-   Routing system to generate routes based on selected truck + trailer and selected tree clusters (#328, #321)
-   Save routing gpx file in s3 storage (#328)

### Changed

-   Refactored sensor entities for better structure (#231).
-   Renamed `wateringPlanStatus` to `status` in watering plan entities (#281).

### Fixed

-   Resolved errors when linking sensors to trees or creating tree clusters (#345, #340, #339).
-   Fixed authentication errors in MQTT handling and API responses (#316, #330).

### Testing

-   Comprehensive test coverage improvements across repositories, services and server layers (#275, #247, #186).
-   Cleanup of test utilities for improved maintainability (#274).

### Removed

-   Removed unused entity types and cleaned up legacy code (#284).

## [v1.0.0] - 2024-10-22

### Added

-   Initial release
-   Create database scheme
-   Implement sqlc to generate type-safe code from SQL (#35)
-   Implement repository pattern for database access for postgres
-   Added seed for demo data (#110)
-   Versioning in API routes (#34)
-   Implement api endpoints for trees, tree cluster, sensor, etc. (#107)
-   Implement handler and service logic for trees, tree cluster, sensor, etc. (#107)
-   Implement user authentication (#37)
-   Implement base structure for user management (#38)
-   Implement logging using slogger (#26)
-   Use Linter to analyze Code
-   Implement import of trees over csv file (#43)
-   Calculate tree cluster center point based on region (#86)
-   Get region by coordinates (#60)
-   Build, test and deploy pipeline

### Fixed

-   Use pgxpool for concurrency-safe connection pool for pgx (#94)

[Unreleased]: https://github.com/green-ecolution/green-ecolution/backend/compare/v1.2.1...HEAD

[v1.2.1]: https://github.com/green-ecolution/green-ecolution/backend/compare/v1.2.0...v1.2.1

[v1.0.0]: https://github.com/green-ecolution/green-ecolution/backend/compare/dfdebe...v1.0.0

[v1.1.0]: https://github.com/green-ecolution/green-ecolution/backend/compare/v1.0.0...v1.1.0

[v1.2.0]: https://github.com/green-ecolution/green-ecolution/backend/compare/v1.1.0...v1.2.0
