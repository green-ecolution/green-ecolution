package vehicle

import (
	"context"
)

type VehicleRepository interface {
	GetAll(ctx context.Context, query VehicleQuery) ([]*Vehicle, int64, error)
	GetCount(ctx context.Context, query VehicleQuery) (int64, error)
	GetByID(ctx context.Context, id int32) (*Vehicle, error)
	GetByPlate(ctx context.Context, plate string) (*Vehicle, error)
	Create(ctx context.Context, entity *Vehicle) (*Vehicle, error)
	Update(ctx context.Context, id int32, entity *Vehicle) error
	Archive(ctx context.Context, id int32) error
	Delete(ctx context.Context, id int32) error
}
