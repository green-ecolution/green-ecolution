package openrouteservice

import (
	"context"
	"errors"
	"io"
	"net/url"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/routing"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/routing/openrouteservice/ors"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/routing/vroom"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

// validate is RouteRepo implements storage.RoutingRepository
var _ storage.RoutingRepository = (*RouteRepo)(nil)

type RouteRepoConfig struct {
	routing config.RoutingConfig
}

type RouteRepo struct {
	vroom vroom.VroomClient
	ors   ors.OrsClient
	cfg   *RouteRepoConfig
}

func NewRouteRepo(cfg *RouteRepoConfig) (*RouteRepo, error) {
	vroomURL, err := url.Parse(cfg.routing.Ors.Optimization.Vroom.Host)
	if err != nil {
		return nil, err
	}
	orsURL, err := url.Parse(cfg.routing.Ors.Host)
	if err != nil {
		return nil, err
	}

	vroomClient := vroom.NewVroomClient(
		vroom.WithHostURL(vroomURL),
		vroom.WithStartPoint(cfg.routing.StartPoint),
		vroom.WithEndPoint(cfg.routing.EndPoint),
		vroom.WithWateringPoint(cfg.routing.WateringPoint),
	)
	orsClient := ors.NewOrsClient(
		ors.WithHostURL(orsURL),
	)

	return &RouteRepo{
		vroom: vroomClient,
		ors:   orsClient,
		cfg:   cfg,
	}, nil
}

func (r *RouteRepo) GenerateRoute(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (*entities.GeoJSON, error) {
	log := logger.GetLogger(ctx)
	orsProfile, err := r.toOrsVehicleType(vehicle.Type)
	if err != nil {
		log.Debug("failed to convert vehicle type to ors vehicle profile", "error", err, "vehicle_type", vehicle.Type)
		return nil, err
	}

	_, orsRoute, err := r.prepareOrsRoute(ctx, vehicle, clusters)
	if err != nil {
		log.Error("failed to prepare route to call ors",
			"error", err,
			"vehicle_id", vehicle.ID,
			"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
		)
		return nil, err
	}

	entity, err := r.ors.DirectionsGeoJSON(ctx, orsProfile, orsRoute)
	if err != nil {
		log.Error("failed to calculate route",
			"error", err,
			"vehicle_id", vehicle.ID,
			"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
		)
		return nil, err
	}

	metadata, err := routing.ConvertLocations(&r.cfg.routing)
	if err != nil {
		log.Error("failed to convert location to route metadata",
			"error", err,
			"vehicle_id", vehicle.ID,
			"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
		)
		return nil, err
	}

	entity.Metadata = *metadata

	log.Debug("route generated successfully",
		"vehicle_id", vehicle.ID,
		"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
	)
	return entity, nil
}

func (r *RouteRepo) GenerateRawGpxRoute(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (io.ReadCloser, error) {
	orsProfile, err := r.toOrsVehicleType(vehicle.Type)
	if err != nil {
		return nil, err
	}

	_, orsRoute, err := r.prepareOrsRoute(ctx, vehicle, clusters)
	if err != nil {
		return nil, err
	}

	return r.ors.DirectionsRawGpx(ctx, orsProfile, orsRoute)
}

func (r *RouteRepo) GenerateRouteInformation(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (*entities.RouteMetadata, error) {
	orsProfile, err := r.toOrsVehicleType(vehicle.Type)
	if err != nil {
		return nil, err
	}

	optimizedRoutes, route, err := r.prepareOrsRoute(ctx, vehicle, clusters)
	if err != nil {
		return nil, err
	}

	var refillCount int
	if len(optimizedRoutes.Routes) > 0 {
		oRoute := optimizedRoutes.Routes[0]
		reducedSteps := utils.Reduce(oRoute.Steps, vroom.ReduceSteps, make([]*vroom.VroomRouteStep, 0, len(oRoute.Steps)))
		refillCount = vroom.RefillCount(reducedSteps)
	}

	rawDirections, err := r.ors.DirectionsJSON(ctx, orsProfile, route)
	if err != nil {
		return nil, err
	}

	var distance, duration float64
	if len(rawDirections.Routes) > 0 {
		distance = rawDirections.Routes[0].Summary.Distance
		duration = rawDirections.Routes[0].Summary.Duration
	}

	return &entities.RouteMetadata{
		Refills:  int32(refillCount),
		Distance: distance,
		Time:     time.Duration(duration * float64(time.Second)),
	}, nil
}

func (r *RouteRepo) prepareOrsRoute(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (optimized *vroom.VroomResponse, routes *ors.OrsDirectionRequest, err error) {
	log := logger.GetLogger(ctx)
	optimizedRoutes, err := r.vroom.OptimizeRoute(ctx, vehicle, clusters)
	if err != nil {
		log.Error("failed to optimize route", "error", err)
		return nil, nil, err
	}

	// currently handle only the first route
	if len(optimizedRoutes.Routes) == 0 {
		log.Error("there are no routes in vroom response", "routes", optimizedRoutes)
		return nil, nil, errors.New("empty routes")
	}
	oRoute := optimizedRoutes.Routes[0]
	reducedSteps := utils.Reduce(oRoute.Steps, vroom.ReduceSteps, make([]*vroom.VroomRouteStep, 0, len(oRoute.Steps)))
	orsLocation := utils.Reduce(reducedSteps, func(acc [][]float64, current *vroom.VroomRouteStep) [][]float64 {
		return append(acc, current.Location)
	}, make([][]float64, 0, len(reducedSteps)))

	return optimizedRoutes, &ors.OrsDirectionRequest{
		Coordinates: orsLocation,
		Units:       "m",
		Language:    "de-de",
	}, nil
}

func (r *RouteRepo) toOrsVehicleType(vehicle entities.VehicleType) (string, error) {
	if vehicle == entities.VehicleTypeUnknown {
		return "", storage.ErrUnknownVehicleType
	}

	return "driving-car", nil // ORS doesn't support dynamic routing over api call
}
