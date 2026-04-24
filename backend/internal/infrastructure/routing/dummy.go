package routing

import (
	"context"
	"io"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/routing"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

// DummyRoutingRepo is used to disable the routing service by configuration
type DummyRoutingRepo struct{}

func NewDummyRoutingRepo() *DummyRoutingRepo {
	return &DummyRoutingRepo{}
}

func (r *DummyRoutingRepo) GenerateRoute(_ context.Context, _, _, _, _ float64, _ []shared.Coordinate) (*routing.GeoJSON, error) {
	return nil, shared.ErrRoutingServiceDisabled
}

func (r *DummyRoutingRepo) GenerateRawGpxRoute(_ context.Context, _, _, _, _ float64, _ []shared.Coordinate) (io.ReadCloser, error) {
	return nil, shared.ErrRoutingServiceDisabled
}

func (r *DummyRoutingRepo) GenerateRouteInformation(_ context.Context, _, _, _, _ float64, _ shared.WaterCapacity, _ []shared.Coordinate, _ []int) (*routing.RouteMetadata, error) {
	return nil, shared.ErrRoutingServiceDisabled
}
