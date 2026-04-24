package application

import (
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/auth"
	"github.com/green-ecolution/green-ecolution/backend/internal/application/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/application/evaluation"
	"github.com/green-ecolution/green-ecolution/backend/internal/application/info"
	"github.com/green-ecolution/green-ecolution/backend/internal/application/plugin"
	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/application/region"
	"github.com/green-ecolution/green-ecolution/backend/internal/application/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/application/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/application/vehicle"
	wateringplan "github.com/green-ecolution/green-ecolution/backend/internal/application/watering"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

func NewService(cfg *config.Config, repos *storage.Repository, eventMananger *worker.EventManager) *ports.Services {
	var authSvc ports.AuthService
	var pluginSvc ports.PluginService
	if cfg.IdentityAuth.Enable {
		authSvc = auth.NewAuthService(repos.Auth, repos.User, &cfg.IdentityAuth)
		pluginSvc = plugin.NewPluginManager(repos.Auth)
	} else {
		slog.Warn("the auth service is disabled due to the configuration")
		authSvc = auth.NewDummyAuthService(repos.User)
		pluginSvc = plugin.NewDummyPluginManager()
	}

	return &ports.Services{
		InfoService:         info.NewInfoService(repos.Info),
		TreeService:         tree.NewTreeService(repos.Tree, repos.Sensor, repos.TreeCluster, eventMananger, cfg.Map),
		AuthService:         authSvc,
		RegionService:       region.NewRegionService(repos.Region),
		TreeClusterService:  cluster.NewTreeClusterService(repos.TreeCluster, repos.Tree, repos.Region, eventMananger),
		VehicleService:      vehicle.NewVehicleService(repos.Vehicle),
		SensorService:       sensor.NewSensorService(repos.Sensor, repos.Tree, eventMananger),
		PluginService:       pluginSvc,
		WateringPlanService: wateringplan.NewWateringPlanService(repos.WateringPlan, repos.TreeCluster, repos.Vehicle, repos.User, eventMananger, repos.Routing, repos.GpxBucket),
		EvaluationService:   evaluation.NewEvaluationService(repos.TreeCluster, repos.Tree, repos.Sensor, repos.WateringPlan, repos.Vehicle),
	}
}
