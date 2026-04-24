package evaluation

import "context"

type EvaluationRepository interface {
	GetRegionsWithWateringPlanCount(ctx context.Context) ([]*RegionEvaluation, error)
	GetVehiclesWithWateringPlanCount(ctx context.Context) ([]*VehicleEvaluation, error)
	GetTotalConsumedWater(ctx context.Context) (int64, error)
	GetWateringPlanUserCount(ctx context.Context) (int64, error)
}
