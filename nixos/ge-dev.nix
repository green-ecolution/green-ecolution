{
  pkgs,
  lib,
  config,
  inputs,
  self,
  ...
}: let
  # ---- DEV SECRETS / PARAMS (Demo-Defaults, bitte anpassen/übersteuern) ----
  POSTGRES_USER = "ge";
  POSTGRES_PASSWORD = "ge";
  POSTGRES_DB = "ge";
  S3_ACCESS_KEY = "minioadmin";
  S3_SECRET_KEY = "minioadmin";
  S3_ROUTE_BUCKET = "routes";
  KC_PASSWORD = "admin";

  GE = {
    SERVER_APP_URL = "http://localhost:3000";
    SERVER_PORT = "3000";
    SERVER_DATABASE_HOST = "127.0.0.1";
    SERVER_DATABASE_PORT = "5432";
    SERVER_DATABASE_TIMEOUT = "30s";
    SERVER_DATABASE_NAME = POSTGRES_DB;
    SERVER_DATABASE_USER = POSTGRES_USER;
    SERVER_DATABASE_PASSWORD = POSTGRES_PASSWORD;

    AUTH_ENABLE = "true";
    AUTH_OIDC_PROVIDER_BASE_URL = "http://auth.localhost:3000";
    AUTH_OIDC_PROVIDER_DOMAIN_NAME = "green-ecolution";
    AUTH_OIDC_PROVIDER_AUTH_URL = "http://auth.localhost:3000/realms/green-ecolution/protocol/openid-connect/auth";
    AUTH_OIDC_PROVIDER_TOKEN_URL = "http://auth.localhost:3000/realms/green-ecolution/protocol/openid-connect/token";
    AUTH_OIDC_PROVIDER_PUBLIC_KEY_STATIC = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAsr2iJerMHstjk88ffPAnqwJ8owifZu1iFzdGeruYU2COAYaIANzj0COEAjEu7qiEL9wusA7QI/N/bBIOiRTJIbiIFF/wv6Jn3xUc0CdZTTTDpeYthDLJQJCdokSW4bBSxtx2T4iONGMwC51Hle7COhJ+ueZnT6aPefVpk4Nel4PmCpAOyy7vvVvwavmXcXKnZmxscF9shThGtzDf1Uq5bocO0QhYQxOy+Gz+ngcqQvCC6bZACJrTru1pDvk66EVYahRBu6cOwYtv2otyP69+LxjEFo+5wlZ6x4oc3kg8Ls48qaJg6POUvNHli/2SZk/l5uTrmDeOIvWUZsYpJB/TBQIDAQAB";
    AUTH_OIDC_PROVIDER_FRONTEND_CLIENT_ID = "frontend";
    AUTH_OIDC_PROVIDER_FRONTEND_CLIENT_SECRET = "frontend-secret";
    AUTH_OIDC_PROVIDER_BACKEND_CLIENT_ID = "backend";
    AUTH_OIDC_PROVIDER_BACKEND_CLIENT_SECRET = "backend-secret";

    ROUTING_ENABLE = "true";
    ROUTING_START_POINT = "9.434764259345679,54.768731253913806";
    ROUTING_END_POINT = "9.434764259345679,54.768731253913806";
    ROUTING_WATERING_POINT = "9.434764259345679,54.768731253913806";
    ROUTING_VALHALLA_HOST = "http://vroom:8002"; # wird via Traefik geroutet
    ROUTING_VALHALLA_OPTIMIZATION_VROOM_HOST = "http://vroom:3000";

    S3_ENABLE = "true";
    S3_ENDPOINT = "127.0.0.1:9000"; # traefik mapped s3.localhost:3000 -> 9000; lokal ok direkt
    S3_REGION = "us-east-1";
    S3_USE_SSL = "false";
    "S3_ROUTE-GPX_BUCKET" = S3_ROUTE_BUCKET;
    "S3_ROUTE-GPX_ACCESSKEY" = S3_ACCESS_KEY;
    "S3_ROUTE-GPX_SECRETACCESSKEY" = S3_SECRET_KEY;

    MQTT_ENABLE = "false";
  };
in {
  ########################################
  # Basis
  ########################################
  networking.hostName = "ge-dev";
  services.openssh.enable = false;
  time.timeZone = "Europe/Berlin";
  users.users.root = {
    # setze /bin/bash als Shell
    shell = pkgs.bashInteractive;
    # Passwort leer lassen (direkter Login ohne Passwort)
    initialPassword = "";
  };

  ########################################
  # Traefik (ersetzt reverse-proxy)
  ########################################
  services.traefik = {
    enable = true;
    staticConfigOptions = {
      api = {insecure = true;};
      entryPoints.web.address = ":3000";
      providers.file.directory = "/etc/traefik/dynamic";
    };
  };

  environment.etc."traefik/dynamic/dynamic.yml".text = ''
    http:
      routers:
        traefik:
          rule: Host(`traefik.localhost`)
          service: api@internal
          entryPoints: [web]
        minio:
          rule: Host(`minio.localhost`)
          service: minio
          entryPoints: [web]
        s3:
          rule: Host(`s3.localhost`)
          service: s3
          entryPoints: [web]
        keycloak:
          rule: Host(`auth.localhost`)
          service: keycloak
          entryPoints: [web]
        pgadmin:
          rule: Host(`pgadmin.localhost`)
          service: pgadmin
          entryPoints: [web]
        vroom:
          rule: Host(`vroom.localhost`)
          service: vroom
          entryPoints: [web]
        valhalla:
          rule: Host(`valhalla.localhost`)
          service: valhalla
          entryPoints: [web]
        backend:
          rule: Host(`localhost`)
          service: backend
          entryPoints: [web]
      services:
        minio:
          loadBalancer: { servers: [ { url: "http://127.0.0.1:9001" } ] }
        s3:
          loadBalancer: { servers: [ { url: "http://127.0.0.1:9000" } ] }
        keycloak:
          loadBalancer: { servers: [ { url: "http://127.0.0.1:3001" } ] }
        pgadmin:
          loadBalancer: { servers: [ { url: "http://127.0.0.1:5050" } ] }
        vroom:
          loadBalancer: { servers: [ { url: "http://127.0.0.1:3002" } ] }
        valhalla:
          loadBalancer: { servers: [ { url: "http://127.0.0.1:8002" } ] }
        backend:
          loadBalancer: { servers: [ { url: "http://127.0.0.1:3000" } ] }
  '';

  ########################################
  # Postgres 17 + PostGIS (ersetzt db:)
  ########################################
  services.postgresql = {
    enable = true;
    package = pkgs.postgresql_17;
    extensions = [pkgs.postgresqlPackages.postgis];
    settings = {
      port = 5432;
      shared_preload_libraries = ["postgis-3"];
    };
    ensureUsers = [
      {
        name = POSTGRES_USER;
        ensureDBOwnership = true;
      }
    ];
    ensureDatabases = [POSTGRES_DB];
    authentication = ''
      local all all trust
      host  all all 127.0.0.1/32 trust
      host  all all ::1/128     trust
    '';
  };
  networking.firewall.allowedTCPPorts = [5432];

  ########################################
  # MinIO (ersetzt storage:)
  ########################################
  services.minio = {
    enable = true;
    listenAddress = "127.0.0.1:9000";
    consoleAddress = "127.0.0.1:9001";
    rootCredentialsFile = "/etc/minio-credentials";
    dataDir = ["/var/lib/minio"];
  };
  environment.etc."minio-credentials".text = "MINIO_ROOT_USER=${S3_ACCESS_KEY}\nMINIO_ROOT_PASSWORD=${S3_SECRET_KEY}\n";

  systemd.services.minio-after = {
    description = "Create default MinIO buckets";
    after = ["minio.service"];
    wants = ["minio.service"];
    serviceConfig.Type = "oneshot";
    script = ''
      set -e
      ${pkgs.curl}/bin/curl -sSf "http://127.0.0.1:9000/minio/health/live" >/dev/null
      ${pkgs.mc}/bin/mc alias set local http://127.0.0.1:9000 ${S3_ACCESS_KEY} ${S3_SECRET_KEY} || true
      ${pkgs.mc}/bin/mc mb --ignore-existing local/${S3_ROUTE_BUCKET}
    '';
  };

  ########################################
  # Keycloak (ersetzt keycloak:)
  ########################################
  services.keycloak = {
    enable = true;
    package = pkgs.keycloak;

    settings = {
      http-enabled = true;
      hostname = "auth.localhost";
      http-port = 3001; # intern; Traefik mapped 3000->3001
    };

    database = {
      type = "postgresql";
      host = "localhost";
      port = 5432;
      username = "keycloak";
      passwordFile = "/etc/kc-db-pass";
      name = "keycloak";
    };

    initialAdminPassword = KC_PASSWORD;

    themes = {};
    realmFiles = [
      ../.docker/infra/keycloak/green-ecolution-realm.json
    ];
  };

  environment.etc."kc-db-pass".text = "${POSTGRES_PASSWORD}\n";

  ########################################
  # pgAdmin / vroom / valhalla (OCI-Container via Podman)
  ########################################
  virtualisation.podman.enable = true;
  virtualisation.oci-containers = {
    backend = "podman";

    containers.pgadmin = {
      image = "docker.io/dpage/pgadmin4:9.6.0";
      ports = ["127.0.0.1:5050:80"];
      environment = {
        PGADMIN_DEFAULT_EMAIL = "1@1.com";
        PGADMIN_DEFAULT_PASSWORD = "1";
      };
    };

    containers.vroom = {
      image = "ghcr.io/vroom-project/vroom-docker:v1.14.0";
      environment = {VROOM_ROUTER = "valhalla";};
      ports = ["127.0.0.1:3002:3000"];
      volumes = ["${toString ../.}/.docker/infra/vroom:/conf:Z"];
    };

    containers.valhalla = {
      image = "ghcr.io/gis-ops/docker-valhalla/valhalla:3.5.1";
      ports = ["127.0.0.1:8002:8002"];
      volumes = ["${toString ../.}/.docker/infra/valhalla/custom_files:/custom_files:Z"];
      environment = {server_threads = "20";};
    };
  };

  ########################################
  # Backend als systemd-Service
  ########################################
  systemd.services.green-ecolution = {
    description = "Green Ecolution";
    after = ["postgresql.service" "minio.service" "keycloak.service"];
    wants = ["postgresql.service" "minio.service" "keycloak.service"];
    wantedBy = ["multi-user.target"];
    serviceConfig = {
      ExecStart = "${self.packages.${pkgs.system}.default}/bin/green-ecolution";
      Restart = "on-failure";
      Environment = [
        "ENV=nixos-dev"
        "GE_SERVER_APP_URL=${GE.SERVER_APP_URL}"
        "GE_SERVER_PORT=${GE.SERVER_PORT}"
        "GE_SERVER_DATABASE_HOST=${GE.SERVER_DATABASE_HOST}"
        "GE_SERVER_DATABASE_PORT=${GE.SERVER_DATABASE_PORT}"
        "GE_SERVER_DATABASE_TIMEOUT=${GE.SERVER_DATABASE_TIMEOUT}"
        "GE_SERVER_DATABASE_NAME=${GE.SERVER_DATABASE_NAME}"
        "GE_SERVER_DATABASE_USER=${GE.SERVER_DATABASE_USER}"
        "GE_SERVER_DATABASE_PASSWORD=${GE.SERVER_DATABASE_PASSWORD}"

        "GE_AUTH_ENABLE=${GE.AUTH_ENABLE}"
        "GE_AUTH_OIDC_PROVIDER_BASE_URL=${GE.AUTH_OIDC_PROVIDER_BASE_URL}"
        "GE_AUTH_OIDC_PROVIDER_DOMAIN_NAME=${GE.AUTH_OIDC_PROVIDER_DOMAIN_NAME}"
        "GE_AUTH_OIDC_PROVIDER_AUTH_URL=${GE.AUTH_OIDC_PROVIDER_AUTH_URL}"
        "GE_AUTH_OIDC_PROVIDER_TOKEN_URL=${GE.AUTH_OIDC_PROVIDER_TOKEN_URL}"
        "GE_AUTH_OIDC_PROVIDER_PUBLIC_KEY_STATIC=${GE.AUTH_OIDC_PROVIDER_PUBLIC_KEY_STATIC}"
        "GE_AUTH_OIDC_PROVIDER_FRONTEND_CLIENT_ID=${GE.AUTH_OIDC_PROVIDER_FRONTEND_CLIENT_ID}"
        "GE_AUTH_OIDC_PROVIDER_FRONTEND_CLIENT_SECRET=${GE.AUTH_OIDC_PROVIDER_FRONTEND_CLIENT_SECRET}"
        "GE_AUTH_OIDC_PROVIDER_BACKEND_CLIENT_ID=${GE.AUTH_OIDC_PROVIDER_BACKEND_CLIENT_ID}"
        "GE_AUTH_OIDC_PROVIDER_BACKEND_CLIENT_SECRET=${GE.AUTH_OIDC_PROVIDER_BACKEND_CLIENT_SECRET}"

        "GE_ROUTING_ENABLE=${GE.ROUTING_ENABLE}"
        "GE_ROUTING_START_POINT=${GE.ROUTING_START_POINT}"
        "GE_ROUTING_END_POINT=${GE.ROUTING_END_POINT}"
        "GE_ROUTING_WATERING_POINT=${GE.ROUTING_WATERING_POINT}"
        "GE_ROUTING_VALHALLA_HOST=${GE.ROUTING_VALHALLA_HOST}"
        "GE_ROUTING_VALHALLA_OPTIMIZATION_VROOM_HOST=${GE.ROUTING_VALHALLA_OPTIMIZATION_VROOM_HOST}"

        "GE_S3_ENABLE=${GE.S3_ENABLE}"
        "GE_S3_ENDPOINT=${GE.S3_ENDPOINT}"
        "GE_S3_REGION=${GE.S3_REGION}"
        "GE_S3_USE_SSL=${GE.S3_USE_SSL}"
        "GE_S3_ROUTE-GPX_BUCKET=${GE."S3_ROUTE-GPX_BUCKET"}"
        "GE_S3_ROUTE-GPX_ACCESSKEY=${GE."S3_ROUTE-GPX_ACCESSKEY"}"
        "GE_S3_ROUTE-GPX_SECRETACCESSKEY=${GE."S3_ROUTE-GPX_SECRETACCESSKEY"}"

        "GE_MQTT_ENABLE=${GE.MQTT_ENABLE}"
      ];
    };
  };

  ########################################
  # Bequeme VM (QEMU) bauen/booten
  ########################################
  virtualisation.vmVariant = {
    virtualisation.forwardPorts = [
      {
        from = "host";
        host.address = "127.0.0.1";
        host.port = 2222;
        guest.port = 22;
      } # ssh
      {
        from = "host";
        host.address = "127.0.0.1";
        host.port = 3000;
        guest.port = 3000;
      } # Traefik
      {
        from = "host";
        host.address = "127.0.0.1";
        host.port = 5432;
        guest.port = 5432;
      } # Postgres direct
      # { from = "host"; host.address = "127.0.0.1"; host.port = 9000; guest.port = 9000; } # MinIO
      # { from = "host"; host.address = "127.0.0.1"; host.port = 9001; guest.port = 9001; } # MinIO console
    ];
  };

  system.stateVersion = "25.11";
}
