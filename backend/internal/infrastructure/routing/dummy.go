package routing

import (
	"context"
	"io"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

// DummyRoutingRepo is used to disable the routing service by configuration
type DummyRoutingRepo struct{}

func NewDummyRoutingRepo() *DummyRoutingRepo {
	return &DummyRoutingRepo{}
}

func (r *DummyRoutingRepo) GenerateRoute(_ context.Context, _ *shared.Vehicle, _ []*shared.TreeCluster) (*shared.GeoJSON, error) {
	return nil, shared.ErrRoutingServiceDisabled
}

func (r *DummyRoutingRepo) GenerateRawGpxRoute(_ context.Context, _ *shared.Vehicle, _ []*shared.TreeCluster) (io.ReadCloser, error) {
	return nil, shared.ErrRoutingServiceDisabled
}

func (r *DummyRoutingRepo) GenerateRouteInformation(_ context.Context, _ *shared.Vehicle, _ []*shared.TreeCluster) (*shared.RouteMetadata, error) {
	return nil, shared.ErrRoutingServiceDisabled
}
