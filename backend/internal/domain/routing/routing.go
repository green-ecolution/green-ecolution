package routing

import (
	"context"
	"io"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type GeoJSONType string // @name GeoJSONType

const (
	FeatureCollection GeoJSONType = "FeatureCollection"
	Feature           GeoJSONType = "Feature"
	LineString        GeoJSONType = "LineString"
)

type GeoJSON struct {
	Type     GeoJSONType      `json:"type"`
	Bbox     []float64        `json:"bbox"`
	Features []GeoJSONFeature `json:"features"`
	Metadata GeoJSONMetadata  `json:"metadata"`
} // @name GeoJSON

type GeoJSONFeature struct {
	Type       GeoJSONType     `json:"type"`
	Bbox       []float64       `json:"bbox"`
	Properties map[string]any  `json:"properties"`
	Geometry   GeoJSONGeometry `json:"geometry"`
} // @name GeoJSONFeature

type GeoJSONGeometry struct {
	Type        GeoJSONType `json:"type"`
	Coordinates [][]float64 `json:"coordinates"`
} // @name GeoJSONGeometry

type GeoJSONMetadata struct {
	StartPoint    GeoJSONLocation
	EndPoint      GeoJSONLocation
	WateringPoint GeoJSONLocation
} // @name GeoJSONMetadata

type GeoJSONLocation struct {
	Coordinate shared.Coordinate
} // @name GeoJSONLocation

type RouteMetadata struct {
	Distance shared.Distance
	Refills  int32
	Time     time.Duration
}

type RoutingRepository interface {
	GenerateRoute(ctx context.Context, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight float64, clusterCoordinates []shared.Coordinate) (*GeoJSON, error)
	GenerateRawGpxRoute(ctx context.Context, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight float64, clusterCoordinates []shared.Coordinate) (io.ReadCloser, error)
	GenerateRouteInformation(ctx context.Context, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight float64, waterCapacity shared.WaterCapacity, clusterCoordinates []shared.Coordinate, treeCounts []int) (*RouteMetadata, error)
}

type S3Repository interface {
	BucketExists(ctx context.Context) (bool, error)
	PutObject(ctx context.Context, objName, contentType string, contentLength int64, r io.Reader) error
	GetObject(ctx context.Context, objName string) (io.ReadSeekCloser, error)
}
