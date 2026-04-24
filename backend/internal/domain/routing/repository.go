package routing

import (
	"context"
	"io"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

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
