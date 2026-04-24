package watering

import (
	"context"

	"github.com/google/uuid"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type WateringPlanRepository interface {
	GetAll(ctx context.Context, query shared.Query) ([]*WateringPlan, int64, error)
	GetCount(ctx context.Context, query shared.Query) (int64, error)
	GetByID(ctx context.Context, id int32) (*WateringPlan, error)
	GetLinkedVehicleIDByIDAndType(ctx context.Context, id int32, vehicleType string) (*int32, error)
	GetLinkedTreeClusterIDsByID(ctx context.Context, id int32) ([]int32, error)
	GetLinkedUsersByID(ctx context.Context, id int32) ([]*uuid.UUID, error)
	GetEvaluationValues(ctx context.Context, id int32) ([]*EvaluationValue, error)
	GetTotalConsumedWater(ctx context.Context) (int64, error)
	GetAllUserCount(ctx context.Context) (int64, error)
	Create(ctx context.Context, entity *WateringPlan) (*WateringPlan, error)
	Update(ctx context.Context, id int32, entity *WateringPlan) error
	Delete(ctx context.Context, id int32) error
}
