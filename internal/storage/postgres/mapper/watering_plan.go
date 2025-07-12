package mapper

import (
	"github.com/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/backend/internal/utils:TimeToTime
// goverter:extend github.com/green-ecolution/backend/internal/utils:StringPtrToString
// goverter:extend github.com/green-ecolution/backend/internal/utils:Float64ToDuration
// goverter:extend MapWateringPlanStatus
type InternalWateringPlanRepoMapper interface {
	// goverter:ignore TreeClusters UserIDs Transporter Trailer Evaluation
	// goverter:map GpxUrl GpxURL
	// goverter:map AdditionalInformations AdditionalInfo | github.com/green-ecolution/backend/internal/utils:MapAdditionalInfo
	FromSql(src *sqlc.WateringPlan) (*entities.WateringPlan, error)
	FromSqlList(src []*sqlc.WateringPlan) ([]*entities.WateringPlan, error)

	EvaluationFromSqlList(src []*sqlc.TreeClusterWateringPlan) []*entities.EvaluationValue
}

func MapWateringPlanStatus(wateringPlanStatus sqlc.WateringPlanStatus) entities.WateringPlanStatus {
	return entities.WateringPlanStatus(wateringPlanStatus)
}
