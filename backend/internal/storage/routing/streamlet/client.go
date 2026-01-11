package streamlet

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"net/http"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/green-ecolution/streamlet/client/go/streamlet"
)

const (
	treeScale = 80 // how much water does a tree need
)

type LatLon struct {
	Lat float64
	Lon float64
}

type StreamletClientConfig struct {
	url    string
	client *http.Client
	depots []LatLon
}

type StreamletClientOption func(*StreamletClientConfig)

type StreamletClient struct {
	cfg    StreamletClientConfig
	client streamlet.ClientWithResponsesInterface
}

func WithClient(client *http.Client) StreamletClientOption {
	return func(cfg *StreamletClientConfig) {
		cfg.client = client
	}
}

func WithHostURL(hostURL string) StreamletClientOption {
	slog.Debug("use streamlet client with host url", "host_url", hostURL)
	return func(cfg *StreamletClientConfig) {
		cfg.url = hostURL
	}
}

func WithDepots(depots []LatLon) StreamletClientOption {
	return func(cfg *StreamletClientConfig) {
		cfg.depots = depots
	}
}

var defaultCfg = StreamletClientConfig{
	client: http.DefaultClient,
	depots: []LatLon{{
		Lat: 54.768326,
		Lon: 9.436183,
	}, {
		Lat: 54.805631,
		Lon: 9.448447,
	}},
}

// func init() {
// 	var truck streamlet.VehicleKind1
// 	truck.Truck.Width = 1.95
// 	truck.Truck.Height = 1.92
// 	truck.Truck.Length = 9.86
// 	truck.Truck.Weight = 6.0
//
// 	if err := vehicle.Kind.FromVehicleKind1(truck); err != nil {
// 		panic(err)
// 	}
// }
//
// var vehicle = streamlet.Vehicle{
// 	Id: 0,
// 	CurrentLoc: streamlet.LatLon{
// 		Lat:  54.768326,
// 		Long: 9.436183,
// 	},
// 	Capacity:    3800,
// 	CurrentLoad: 3800,
// 	CurrentTime: 0,
// }
//
// var depots = []streamlet.Depot{
// 	{
// 		Id: 1,
// 		Loc: streamlet.LatLon{
// 			Lat:  54.768326,
// 			Long: 9.436183,
// 		},
// 	},
//
// 	{
// 		Id: 2,
// 		Loc: streamlet.LatLon{
// 			Lat:  54.805631,
// 			Long: 9.448447,
// 		},
// 	},
// }

var _ storage.RoutingRepository = (*StreamletClient)(nil)

func NewStreamletClient(opts ...StreamletClientOption) (*StreamletClient, error) {
	cfg := defaultCfg
	for _, opt := range opts {
		opt(&cfg)
	}

	fmt.Printf("%+v\n", cfg)

	client, err := streamlet.NewClientWithResponses(cfg.url)
	if err != nil {
		return nil, err
	}

	return &StreamletClient{
		cfg:    cfg,
		client: client,
	}, nil
}

func (r *StreamletClient) GenerateRoute(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (*entities.GeoJSON, error) {
	log := logger.GetLogger(ctx)

	depots := utils.MapIdx(r.cfg.depots, func(d LatLon, idx int) streamlet.Depot {
		return streamlet.Depot{
			Id: uint32(idx + 1),
			Loc: streamlet.LatLon{
				Lat:  d.Lat,
				Long: d.Lon,
			},
		}
	})

	customers, err := utils.MapError(clusters, func(c *entities.TreeCluster) (streamlet.Customer, error) {
		if c.Latitude == nil || c.Longitude == nil {
			return streamlet.Customer{}, fmt.Errorf("found nil values in coordinates of treecluster %d", c.ID)
		}
		return streamlet.Customer{
			Id:     uint32(c.ID),
			Demand: float32(len(c.Trees) * treeScale),
			Loc: streamlet.LatLon{
				Lat:  *c.Latitude,
				Long: *c.Longitude,
			},
		}, nil
	})
	if err != nil {
		log.Error("failed to map treeclusters to client", "error", err,
			"vehicle_id", vehicle.ID,
			"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
		)
		return nil, err
	}

	sVehicle := streamlet.Vehicle{
		Id:          0,
		CurrentTime: 0.0,
		Capacity:    float32(vehicle.WaterCapacity),
		CurrentLoad: float32(vehicle.WaterCapacity),
		CurrentLoc: streamlet.LatLon{
			Lat:  r.cfg.depots[0].Lat,
			Long: r.cfg.depots[0].Lon,
		},
	}

	var truck streamlet.VehicleKind1
	truck.Truck.Height = float32(vehicle.Height)
	truck.Truck.Width = float32(vehicle.Width)
	truck.Truck.Length = float32(vehicle.Length)
	truck.Truck.Weight = float32(vehicle.Weight)
	if err := sVehicle.Kind.FromVehicleKind1(truck); err != nil {
		log.Error("failed to map vehicle to streamlet Vehicle", "error", err,
			"vehicle_id", vehicle.ID,
			"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
		)
		return nil, err
	}

	res, err := r.client.SolveAndCalculateRouteWithResponse(ctx, "valhalla", streamlet.SolveRequest{
		Depots:  depots,
		Clients: customers,
		Vehicle: sVehicle,
	})
	if err != nil {
		log.Error("failed to solve and calculate route in streamlet", "error", err,
			"vehicle_id", vehicle.ID,
			"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
		)
		return nil, err
	}

	if res.StatusCode() != http.StatusOK {
		err = fmt.Errorf("status code not ok: %s: %s", res.Status(), string(res.Body))
		log.Error("failed to calculate route from streamlet", "error", err,
			"vehicle_id", vehicle.ID,
			"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
		)
		return nil, err
	}

	geometry := res.JSON200.Data.Routes[0].Geometry

	log.Debug("route generated successfully",
		"vehicle_id", vehicle.ID,
		"clusters_ids", utils.Map(clusters, func(c *entities.TreeCluster) int32 { return c.ID }),
	)

	return &entities.GeoJSON{
		Type: entities.FeatureCollection,
		Features: []entities.GeoJSONFeature{
			{
				Type: entities.Feature,
				Geometry: entities.GeoJSONGeometry{
					Type:        entities.LineString,
					Coordinates: geometry.Coordinates,
				},
			},
		},
	}, nil
}

func (r *StreamletClient) GenerateRawGpxRoute(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (io.ReadCloser, error) {
	return nil, fmt.Errorf("Not implemented")
}

func (r *StreamletClient) GenerateRouteInformation(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (*entities.RouteMetadata, error) {
	return nil, fmt.Errorf("Not implemented")
}
