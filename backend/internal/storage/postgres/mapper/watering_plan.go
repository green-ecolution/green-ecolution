package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalWateringPlanRepoMapper interface {
	FromSql(src *sqlc.WateringPlan) (*entities.WateringPlan, error)
	FromSqlList(src []*sqlc.WateringPlan) ([]*entities.WateringPlan, error)
	EvaluationFromSqlList(src []*sqlc.TreeClusterWateringPlan) []*entities.EvaluationValue
}

type InternalWateringPlanRepoMapperImpl struct{}

func (c *InternalWateringPlanRepoMapperImpl) FromSql(source *sqlc.WateringPlan) (*entities.WateringPlan, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	result := &entities.WateringPlan{
		ID:               source.ID,
		CreatedAt:        utils.TimeToTime(source.CreatedAt),
		UpdatedAt:        utils.TimeToTime(source.UpdatedAt),
		Date:             utils.TimeToTime(source.Date),
		Description:      source.Description,
		Status:           MapWateringPlanStatus(source.Status),
		CancellationNote: source.CancellationNote,
		GpxURL:           utils.StringPtrToString(source.GpxUrl),
		RefillCount:      source.RefillCount,
		Duration:         utils.Float64ToDuration(source.Duration),
		Provider:         utils.StringPtrToString(source.Provider),
		AdditionalInfo:   additionalInfo,
	}
	if source.Distance != nil {
		v := *source.Distance
		result.Distance = &v
	}
	if source.TotalWaterRequired != nil {
		v := *source.TotalWaterRequired
		result.TotalWaterRequired = &v
	}
	return result, nil
}

func (c *InternalWateringPlanRepoMapperImpl) FromSqlList(source []*sqlc.WateringPlan) ([]*entities.WateringPlan, error) {
	return utils.MapSliceErr(source, c.FromSql)
}

func (c *InternalWateringPlanRepoMapperImpl) EvaluationFromSqlList(source []*sqlc.TreeClusterWateringPlan) []*entities.EvaluationValue {
	return utils.MapSlice(source, evaluationFromSql)
}

func evaluationFromSql(source *sqlc.TreeClusterWateringPlan) *entities.EvaluationValue {
	if source == nil {
		return nil
	}
	v := source.ConsumedWater
	return &entities.EvaluationValue{
		WateringPlanID: source.WateringPlanID,
		TreeClusterID:  source.TreeClusterID,
		ConsumedWater:  &v,
	}
}

func MapWateringPlanStatus(wateringPlanStatus sqlc.WateringPlanStatus) entities.WateringPlanStatus {
	return entities.WateringPlanStatus(wateringPlanStatus)
}
