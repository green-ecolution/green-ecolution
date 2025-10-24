package routing

import (
	"context"
	"io"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/storage"
)

// DummyRoutingRepo is used to disable the routing service by configuration
type DummyRoutingRepo struct{}

func NewDummyRoutingRepo() *DummyRoutingRepo {
	return &DummyRoutingRepo{}
}

func (r *DummyRoutingRepo) GenerateRoute(_ context.Context, _ *entities.Vehicle, _ []*entities.TreeCluster) (*entities.GeoJSON, error) {
	return nil, storage.ErrRoutingServiceDisabled
}

func (r *DummyRoutingRepo) GenerateRawGpxRoute(_ context.Context, _ *entities.Vehicle, _ []*entities.TreeCluster) (io.ReadCloser, error) {
	return nil, storage.ErrRoutingServiceDisabled
}

func (r *DummyRoutingRepo) GenerateRouteInformation(_ context.Context, _ *entities.Vehicle, _ []*entities.TreeCluster) (*entities.RouteMetadata, error) {
	return nil, storage.ErrRoutingServiceDisabled
}
