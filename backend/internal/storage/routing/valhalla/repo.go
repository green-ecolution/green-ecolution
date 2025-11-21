package valhalla

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
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/routing/valhalla/valhalla"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/routing/vroom"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

// validate is RouteRepo implements storage.RoutingRepository
var _ storage.RoutingRepository = (*RouteRepo)(nil)

type RouteRepoConfig struct {
	routing config.RoutingConfig
}

type RouteRepo struct {
	vroom    vroom.VroomClient
	valhalla valhalla.ValhallaClient
	cfg      *RouteRepoConfig
}

func NewRouteRepo(cfg *RouteRepoConfig) (*RouteRepo, error) {
	vroomURL, err := url.Parse(cfg.routing.Valhalla.Optimization.Vroom.Host)
	if err != nil {
		return nil, err
	}
	valhallaURL, err := url.Parse(cfg.routing.Valhalla.Host)
	if err != nil {
		return nil, err
	}

	vroomClient := vroom.NewVroomClient(
		vroom.WithHostURL(vroomURL),
		vroom.WithStartPoint(cfg.routing.StartPoint),
		vroom.WithEndPoint(cfg.routing.EndPoint),
		vroom.WithWateringPoint(cfg.routing.WateringPoint),
	)
	valhalllaClient := valhalla.NewValhallaClient(
		valhalla.WithHostURL(valhallaURL),
	)

	return &RouteRepo{
		vroom:    vroomClient,
		valhalla: valhalllaClient,
		cfg:      cfg,
	}, nil
}

func (r *RouteRepo) GenerateRoute(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (*entities.GeoJSON, error) {
	log := logger.GetLogger(ctx)
	_, route, err := r.prepareRoute(ctx, vehicle, clusters)
	if err != nil {
		log.Error("failed to prepare route", "error", err,
			"vehicle_id", vehicle.ID,
			"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
		)
		return nil, err
	}

	entity, err := r.valhalla.DirectionsGeoJSON(ctx, route)
	if err != nil {
		log.Error("failed to generate route in valhalla", "error", err,
			"vehicle_id", vehicle.ID,
			"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
		)
		return nil, err
	}

	metadata, err := routing.ConvertLocations(&r.cfg.routing)
	if err != nil {
		log.Error("failed to convert generated locations", "error", err,
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
	log := logger.GetLogger(ctx)
	_, route, err := r.prepareRoute(ctx, vehicle, clusters)
	if err != nil {
		return nil, err
	}

	log.Debug("route generated successfully as gpx file",
		"vehicle_id", vehicle.ID,
		"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
	)
	return r.valhalla.DirectionsRawGpx(ctx, route)
}

func (r *RouteRepo) GenerateRouteInformation(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (*entities.RouteMetadata, error) {
	optimizedRoutes, route, err := r.prepareRoute(ctx, vehicle, clusters)
	if err != nil {
		return nil, err
	}

	// currently handle only the first route
	var refillCount int
	if len(optimizedRoutes.Routes) > 0 {
		oRoute := optimizedRoutes.Routes[0]
		reducedSteps := utils.Reduce(oRoute.Steps, vroom.ReduceSteps, make([]*vroom.VroomRouteStep, 0, len(oRoute.Steps)))
		refillCount = vroom.RefillCount(reducedSteps)
	}

	rawDirections, err := r.valhalla.DirectionsJSON(ctx, route)
	if err != nil {
		return nil, err
	}

	return &entities.RouteMetadata{
		Refills:  int32(refillCount),
		Distance: rawDirections.Trip.Summary.Length,
		Time:     time.Duration(rawDirections.Trip.Summary.Time * float64(time.Second)),
	}, nil
}

func (r *RouteRepo) prepareRoute(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (optimized *vroom.VroomResponse, routes *valhalla.DirectionRequest, err error) {
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
	locations := utils.Map(reducedSteps, func(step *vroom.VroomRouteStep) valhalla.Location {
		return valhalla.Location{
			Lat:  step.Location[1],
			Lon:  step.Location[0],
			Type: "break",
		}
	})

	costingOpts := make(map[string]valhalla.CostingOptions)
	costingOpts["truck"] = valhalla.CostingOptions{
		Width:     vehicle.Width,
		Height:    vehicle.Height,
		Length:    vehicle.Length,
		Weight:    vehicle.Weight,
		AxleLoad:  0.0,
		AxleCount: 2,
	}

	return optimizedRoutes, &valhalla.DirectionRequest{
		Locations:      locations,
		Costing:        "truck",
		CostingOptions: costingOpts,
	}, nil
}
