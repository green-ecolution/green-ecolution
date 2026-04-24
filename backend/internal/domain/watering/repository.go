package watering

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type WateringPlanRepository interface {
	GetAll(ctx context.Context, query shared.Query) ([]*WateringPlan, int64, error)
	GetCount(ctx context.Context, query shared.Query) (int64, error)
	GetByID(ctx context.Context, id int32) (*WateringPlan, error)
	Create(ctx context.Context, entity *WateringPlan) (*WateringPlan, error)
	Update(ctx context.Context, id int32, entity *WateringPlan) error
	Delete(ctx context.Context, id int32) error
}
