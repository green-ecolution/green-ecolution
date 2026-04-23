package routing

import (
	"context"
	"io"

	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

// DummyRoutingRepo is used to disable the routing service by configuration
type DummyRoutingRepo struct{}

func NewDummyRoutingRepo() *DummyRoutingRepo {
	return &DummyRoutingRepo{}
}

func (r *DummyRoutingRepo) GenerateRoute(_ context.Context, _ *entities.Vehicle, _ []*entities.TreeCluster) (*entities.GeoJSON, error) {
	return nil, entities.ErrRoutingServiceDisabled
}

func (r *DummyRoutingRepo) GenerateRawGpxRoute(_ context.Context, _ *entities.Vehicle, _ []*entities.TreeCluster) (io.ReadCloser, error) {
	return nil, entities.ErrRoutingServiceDisabled
}

func (r *DummyRoutingRepo) GenerateRouteInformation(_ context.Context, _ *entities.Vehicle, _ []*entities.TreeCluster) (*entities.RouteMetadata, error) {
	return nil, entities.ErrRoutingServiceDisabled
}
