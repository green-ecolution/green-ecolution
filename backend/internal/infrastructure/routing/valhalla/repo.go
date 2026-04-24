package valhalla

import (
	"context"
	"errors"
	"io"
	"net/url"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/routing"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	infraRouting "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/routing"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/routing/valhalla/valhalla"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/routing/vroom"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

// validate is RouteRepo implements routing.RoutingRepository
var _ routing.RoutingRepository = (*RouteRepo)(nil)

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

func (r *RouteRepo) GenerateRoute(ctx context.Context, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight float64, clusterCoordinates []shared.Coordinate) (*routing.GeoJSON, error) {
	log := logger.GetLogger(ctx)
	_, route, err := r.prepareRoute(ctx, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight, clusterCoordinates)
	if err != nil {
		log.Error("failed to prepare route", "error", err)
		return nil, err
	}

	entity, err := r.valhalla.DirectionsGeoJSON(ctx, route)
	if err != nil {
		log.Error("failed to generate route in valhalla", "error", err)
		return nil, err
	}

	metadata, err := infraRouting.ConvertLocations(&r.cfg.routing)
	if err != nil {
		log.Error("failed to convert generated locations", "error", err)
		return nil, err
	}

	entity.Metadata = *metadata

	log.Debug("route generated successfully")
	return entity, nil
}

func (r *RouteRepo) GenerateRawGpxRoute(ctx context.Context, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight float64, clusterCoordinates []shared.Coordinate) (io.ReadCloser, error) {
	log := logger.GetLogger(ctx)
	_, route, err := r.prepareRoute(ctx, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight, clusterCoordinates)
	if err != nil {
		return nil, err
	}

	log.Debug("route generated successfully as gpx file")
	return r.valhalla.DirectionsRawGpx(ctx, route)
}

func (r *RouteRepo) GenerateRouteInformation(ctx context.Context, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight float64, waterCapacity shared.WaterCapacity, clusterCoordinates []shared.Coordinate, treeCounts []int) (*routing.RouteMetadata, error) {
	optimizedRoutes, route, err := r.prepareRoute(ctx, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight, clusterCoordinates)
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

	return &routing.RouteMetadata{
		Refills:  int32(refillCount),
		Distance: shared.MustNewDistance(rawDirections.Trip.Summary.Length),
		Time:     time.Duration(rawDirections.Trip.Summary.Time * float64(time.Second)),
	}, nil
}

func (r *RouteRepo) prepareRoute(ctx context.Context, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight float64, clusterCoordinates []shared.Coordinate) (optimized *vroom.VroomResponse, routes *valhalla.DirectionRequest, err error) {
	log := logger.GetLogger(ctx)
	// Pass zero water capacity and empty tree counts since vroom optimization
	// only needs coordinates for the route; capacity is not used here.
	optimizedRoutes, err := r.vroom.OptimizeRoute(ctx, shared.MustNewWaterCapacity(0), clusterCoordinates, nil)
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
		Width:     vehicleWidth,
		Height:    vehicleHeight,
		Length:    vehicleLength,
		Weight:    vehicleWeight,
		AxleLoad:  0.0,
		AxleCount: 2,
	}

	return optimizedRoutes, &valhalla.DirectionRequest{
		Locations:      locations,
		Costing:        "truck",
		CostingOptions: costingOpts,
	}, nil
}
