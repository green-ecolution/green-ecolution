package storage

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/auth"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/evaluation"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/info"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/region"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/routing"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/user"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/watering"
)

type Repository struct {
	Auth         auth.AuthRepository
	Info         info.InfoRepository
	Sensor       sensor.SensorRepository
	Tree         tree.TreeRepository
	User         user.UserRepository
	Vehicle      vehicle.VehicleRepository
	TreeCluster  cluster.TreeClusterRepository
	Region       region.RegionRepository
	WateringPlan watering.WateringPlanRepository
	Routing      routing.RoutingRepository
	GpxBucket    routing.S3Repository
	Evaluation   evaluation.EvaluationRepository
}
