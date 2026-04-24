package vehicle

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/evaluation"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type VehicleRepository interface {
	GetAll(ctx context.Context, query shared.Query) ([]*Vehicle, int64, error)
	GetCount(ctx context.Context, query shared.Query) (int64, error)
	GetAllWithArchived(ctx context.Context, provider string) ([]*Vehicle, int64, error)
	GetAllByType(ctx context.Context, provider string, vehicleType VehicleType) ([]*Vehicle, int64, error)
	GetAllByTypeWithArchived(ctx context.Context, provider string, vehicleType VehicleType) ([]*Vehicle, int64, error)
	GetAllArchived(ctx context.Context) ([]*Vehicle, error)
	GetAllWithWateringPlanCount(ctx context.Context) ([]*evaluation.VehicleEvaluation, error)
	GetByID(ctx context.Context, id int32) (*Vehicle, error)
	GetByPlate(ctx context.Context, plate string) (*Vehicle, error)
	Create(ctx context.Context, entity *Vehicle) (*Vehicle, error)
	Update(ctx context.Context, id int32, entity *Vehicle) error
	Archive(ctx context.Context, id int32) error
	Delete(ctx context.Context, id int32) error
}
