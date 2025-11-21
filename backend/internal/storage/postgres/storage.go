package postgres

import (
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	mapper "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/mapper/generated"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/region"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/treecluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/vehicle"
	wateringplan "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/watering_plan"
	"github.com/jackc/pgx/v5/pgxpool"
)

func NewRepository(conn *pgxpool.Pool) *storage.Repository {
	treeMappers := tree.NewTreeRepositoryMappers(
		&mapper.InternalTreeRepoMapperImpl{},
		&mapper.InternalSensorRepoMapperImpl{},
		&mapper.InternalTreeClusterRepoMapperImpl{},
	)
	treeRepo := tree.NewTreeRepository(store.NewStore(conn, sqlc.New(conn)), treeMappers)
	slog.Info("successfully initialized tree repository", "service", "postgres")

	tcMappers := treecluster.NewTreeClusterRepositoryMappers(
		&mapper.InternalTreeClusterRepoMapperImpl{},
		&mapper.InternalSensorRepoMapperImpl{},
		&mapper.InternalRegionRepoMapperImpl{},
		&mapper.InternalTreeRepoMapperImpl{},
	)
	treeClusterRepo := treecluster.NewTreeClusterRepository(store.NewStore(conn, sqlc.New(conn)), tcMappers)
	slog.Info("successfully initialized treecluster repository", "service", "postgres")

	vehicleMappers := vehicle.NewVehicleRepositoryMappers(
		&mapper.InternalVehicleRepoMapperImpl{},
	)
	vehicleRepo := vehicle.NewVehicleRepository(store.NewStore(conn, sqlc.New(conn)), vehicleMappers)
	slog.Info("successfully initialized vehicle repository", "service", "postgres")

	sensorMappers := sensor.NewSensorRepositoryMappers(
		&mapper.InternalSensorRepoMapperImpl{},
	)
	sensorRepo := sensor.NewSensorRepository(store.NewStore(conn, sqlc.New(conn)), sensorMappers)
	slog.Info("successfully initialized sensor repository", "service", "postgres")

	regionMappers := region.NewRegionMappers(
		&mapper.InternalRegionRepoMapperImpl{},
	)
	regionRepo := region.NewRegionRepository(store.NewStore(conn, sqlc.New(conn)), regionMappers)
	slog.Info("successfully initialized region repository", "service", "postgres")

	wateringPlanMappers := wateringplan.NewWateringPlanRepositoryMappers(
		&mapper.InternalWateringPlanRepoMapperImpl{},
		&mapper.InternalVehicleRepoMapperImpl{},
		&mapper.InternalTreeClusterRepoMapperImpl{},
	)
	wateringPlanRepo := wateringplan.NewWateringPlanRepository(store.NewStore(conn, sqlc.New(conn)), wateringPlanMappers)
	slog.Info("successfully initialized wateringplan repository", "service", "postgres")

	return &storage.Repository{
		Tree:         treeRepo,
		TreeCluster:  treeClusterRepo,
		Vehicle:      vehicleRepo,
		Sensor:       sensorRepo,
		Region:       regionRepo,
		WateringPlan: wateringPlanRepo,
	}
}
