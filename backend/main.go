//go:generate go tool mockery
//go:generate go tool swag fmt
//go:generate go tool swag init --requiredByDefault
//go:generate go tool goverter gen github.com/green-ecolution/backend/internal/server/http/entities/...
//go:generate go tool goverter gen github.com/green-ecolution/backend/internal/server/mqtt/entities/...
//go:generate go tool goverter gen github.com/green-ecolution/backend/internal/storage/mongodb/entities/...
//go:generate go tool goverter gen github.com/green-ecolution/backend/internal/storage/postgres/mapper/...
package main

import (
	"context"
	"embed"
	"fmt"
	"log"
	"log/slog"
	"os"
	"os/signal"
	"strings"
	"sync"
	"syscall"

	"github.com/green-ecolution/backend/docs"
	"github.com/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/server/http"
	"github.com/green-ecolution/backend/internal/server/mqtt"
	"github.com/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/backend/internal/service/domain"
	"github.com/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/backend/internal/storage/auth"
	"github.com/green-ecolution/backend/internal/storage/local"
	"github.com/green-ecolution/backend/internal/storage/postgres"
	"github.com/green-ecolution/backend/internal/storage/routing"
	_ "github.com/green-ecolution/backend/internal/storage/routing/openrouteservice"
	"github.com/green-ecolution/backend/internal/storage/routing/valhalla"
	"github.com/green-ecolution/backend/internal/storage/s3"
	"github.com/green-ecolution/backend/internal/worker"
	"github.com/green-ecolution/backend/internal/worker/subscriber"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/spf13/viper"
	"github.com/twpayne/go-geos"
	pgxgeos "github.com/twpayne/pgx-geos"
)

//go:embed all:frontend
var frontendFS embed.FS

var version = "develop"

//	@title			Green Space Management API
//	@version		develop
//	@description	This is the API for the Green Ecolution Management System.
//	@termsOfService	http://swagger.io/terms/

//	@contact.name	Green Ecolution
//	@contact.url	https://green-ecolution.de
//	@contact.email	info@green-ecolution.de

//	@license.name	AGPL
//	@license.url	https://raw.githubusercontent.com/green-ecolution/green-ecolution/develop/LICENSE

// @securitydefinitions.oauth2.accessCode	Keycloak
// @tokenUrl								https://auth.green-ecolution.de/realms/green-ecolution-dev/protocol/openid-connect/token
// @authorizationUrl						https://auth.green-ecolution.de/realms/green-ecolution-dev/protocol/openid-connect/auth
// @in										header
// @name									Authorization
func main() {
	cfg, err := config.InitConfig()
	if err != nil {
		log.Fatal(err)
	}

	if cfg.Server.Development {
		cfg.Server.Logs.Level = "debug"
	}

	logg := logger.CreateLogger(os.Stdout, "console", cfg.Server.Logs.Level)
	slog.SetDefault(logg())

	osEnv := os.Getenv("ENV")
	slog.Info("starting green ecolution Server", "version", version, "debug_mode", cfg.Server.Development, "env", osEnv)

	setSwaggerInfo(cfg.Server.AppURL)

	ctx, cancel := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer cancel()

	startAppServices(ctx, cfg)
}

func postgresRepo(ctx context.Context, cfg *config.Config) (repo *storage.Repository, closeFn func()) {
	dbCfg := cfg.Server.Database
	slog.Info("try to connect to PostgreSQL database")
	slog.Debug("try to connect to PostgreSQL database with the current configurations", "host", dbCfg.Host, "port", dbCfg.Port, "db_name", dbCfg.Name, "user", dbCfg.Username, "password", "*******")
	connStr := fmt.Sprintf("host=%s port=%d user=%s password=%s dbname=%s sslmode=disable", dbCfg.Host, dbCfg.Port, dbCfg.Username, dbCfg.Password, dbCfg.Name)

	pgxConfig, err := pgxpool.ParseConfig(connStr)
	if err != nil {
		slog.Error("error while parsing PostgreSQL connection string", "error", err, "connection_string", connStr)
		panic(err)
	}

	pgxConfig.AfterConnect = func(ctx context.Context, conn *pgx.Conn) error {
		return pgxgeos.Register(ctx, conn, geos.NewContext())
	}

	pool, err := pgxpool.NewWithConfig(ctx, pgxConfig)
	if err != nil {
		slog.Error("error while connecting to PostgreSQL", "error", err)
		panic(err)
	}

	return postgres.NewRepository(pool), pool.Close
}

func startAppServices(ctx context.Context, cfg *config.Config) {
	repositories, closeFn := initializeRepositories(ctx, cfg)
	defer closeFn()

	em := initializeEventManager()

	services := domain.NewService(cfg, repositories, em)
	httpServer := http.NewServer(cfg, services, frontendFS)
	mqttServer := mqtt.NewMqtt(cfg, services)

	runServices(ctx, httpServer, mqttServer, em, services)
}

func initializeRepositories(ctx context.Context, cfg *config.Config) (repos *storage.Repository, closeFn func()) {
	postgresRepo, closeFn := postgresRepo(ctx, cfg)
	localRepo, err := local.NewRepository(cfg)
	if err != nil {
		panic(err)
	}

	// can be switched between ors and valhalla
	// routingRepo, err := openrouteservice.NewRepository(cfg)
	var routingRepo *storage.Repository
	if cfg.Routing.Enable {
		routingRepo, err = valhalla.NewRepository(cfg)
		if err != nil {
			panic(err)
		}
	} else {
		slog.Warn("the routing service is disabled due to the configuration")
		routingRepo = &storage.Repository{
			Routing: routing.NewDummyRoutingRepo(),
		}
	}

	keycloakRepo := auth.NewRepository(&cfg.IdentityAuth)

	var s3Repos *storage.Repository
	if viper.GetBool("s3.enable") {
		s3Repos, err = s3.NewRepository(cfg)
		if err != nil {
			panic(err)
		}
	} else {
		slog.Warn("the s3 service is disabled due to the configuration")
		s3Repos = &storage.Repository{
			GpxBucket: s3.NewS3DummyRepo(),
		}
	}

	repositories := &storage.Repository{
		Auth: keycloakRepo.Auth,
		User: keycloakRepo.User,

		Info:         localRepo.Info,
		Sensor:       postgresRepo.Sensor,
		Tree:         postgresRepo.Tree,
		TreeCluster:  postgresRepo.TreeCluster,
		Vehicle:      postgresRepo.Vehicle,
		Region:       postgresRepo.Region,
		WateringPlan: postgresRepo.WateringPlan,
		Routing:      routingRepo.Routing,
		GpxBucket:    s3Repos.GpxBucket,
	}

	return repositories, closeFn
}

func initializeEventManager() *worker.EventManager {
	return worker.NewEventManager(
		entities.EventTypeUpdateTree,
		entities.EventTypeUpdateTreeCluster,
		entities.EventTypeCreateTree,
		entities.EventTypeDeleteTree,
		entities.EventTypeNewSensorData,
		entities.EventTypeUpdateWateringPlan,
	)
}

func runServices(ctx context.Context, httpServer *http.Server, mqttServer *mqtt.Mqtt, em *worker.EventManager, services *service.Services) {
	var wg sync.WaitGroup

	if viper.GetBool("mqtt.enable") {
		wg.Add(1)
		go func() {
			defer wg.Done()
			mqttServer.RunSubscriber(ctx)
		}()
	} else {
		slog.Warn("the mqtt service is disabled due to the configuration")
	}

	wg.Add(1)
	go func() {
		defer wg.Done()
		em.Run(ctx)
	}()

	runEventSubscriptions(ctx, &wg, em, services)

	wg.Add(1)
	go func() {
		defer wg.Done()
		if err := httpServer.Run(ctx); err != nil {
			slog.Error("Error while running HTTP Server", "error", err)
		}
	}()

	wg.Wait()
}

func runEventSubscriptions(ctx context.Context, wg *sync.WaitGroup, em *worker.EventManager, services *service.Services) {
	subscribers := []worker.Subscriber{
		subscriber.NewUpdateTreeSubscriber(services.TreeClusterService),
		subscriber.NewCreateTreeSubscriber(services.TreeClusterService),
		subscriber.NewDeleteTreeSubscriber(services.TreeClusterService),
		subscriber.NewSensorDataSubscriber(services.TreeClusterService, services.TreeService),
		subscriber.NewUpdateWateringPlanSubscriber(services.TreeClusterService),
	}

	for _, sub := range subscribers {
		wg.Add(1)
		go func(sub worker.Subscriber) {
			defer wg.Done()
			if err := em.RunSubscription(ctx, sub); err != nil {
				slog.Error("stop subscription with err", "eventType", sub.EventType(), "err", err)
			}
		}(sub)
	}
}

func setSwaggerInfo(appURL string) {
	title := "Green Ecolution Management API"
	description := "This is the API for the Green Ecolution Management System."
	basePath := "/api"

	var schemes []string
	var trimmedAppURL string
	if strings.HasPrefix(appURL, "http://") {
		schemes = []string{"http"}
		trimmedAppURL = strings.TrimPrefix(appURL, "http://")
	} else {
		trimmedAppURL = strings.TrimPrefix(appURL, "https://")
		schemes = []string{"https"}
	}

	docs.SwaggerInfo.Title = title
	docs.SwaggerInfo.Version = version
	docs.SwaggerInfo.Description = description
	docs.SwaggerInfo.Host = trimmedAppURL
	docs.SwaggerInfo.BasePath = basePath
	docs.SwaggerInfo.Schemes = schemes

	slog.Info("setting up swagger docs", "app_url", trimmedAppURL, "title", title, "description", description, "base_path", basePath, "schemes", schemes)
}
