package region

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type RegionRepository interface {
	GetAll(ctx context.Context) ([]*Region, int64, error)
	GetByID(ctx context.Context, id int32) (*Region, error)
	GetByPoint(ctx context.Context, coord shared.Coordinate) (*Region, error)
	Create(ctx context.Context, entity *Region) (*Region, error)
	Update(ctx context.Context, id int32, entity *Region) (*Region, error)
	Delete(ctx context.Context, id int32) error
}
