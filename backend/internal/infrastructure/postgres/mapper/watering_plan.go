package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalWateringPlanRepoMapper interface {
	FromSql(src *sqlc.WateringPlan) (*shared.WateringPlan, error)
	FromSqlList(src []*sqlc.WateringPlan) ([]*shared.WateringPlan, error)
	EvaluationFromSqlList(src []*sqlc.TreeClusterWateringPlan) []*shared.EvaluationValue
}

type InternalWateringPlanRepoMapperImpl struct{}

func (c *InternalWateringPlanRepoMapperImpl) FromSql(source *sqlc.WateringPlan) (*shared.WateringPlan, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	result := &shared.WateringPlan{
		ID:               source.ID,
		CreatedAt:        source.CreatedAt,
		UpdatedAt:        source.UpdatedAt,
		Date:             source.Date,
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
		d := shared.MustNewDistance(*source.Distance)
		result.Distance = &d
	}
	if source.TotalWaterRequired != nil {
		v := *source.TotalWaterRequired
		result.TotalWaterRequired = &v
	}
	return result, nil
}

func (c *InternalWateringPlanRepoMapperImpl) FromSqlList(source []*sqlc.WateringPlan) ([]*shared.WateringPlan, error) {
	return utils.MapSliceErr(source, c.FromSql)
}

func (c *InternalWateringPlanRepoMapperImpl) EvaluationFromSqlList(source []*sqlc.TreeClusterWateringPlan) []*shared.EvaluationValue {
	return utils.MapSlice(source, evaluationFromSql)
}

func evaluationFromSql(source *sqlc.TreeClusterWateringPlan) *shared.EvaluationValue {
	if source == nil {
		return nil
	}
	v := source.ConsumedWater
	return &shared.EvaluationValue{
		WateringPlanID: source.WateringPlanID,
		TreeClusterID:  source.TreeClusterID,
		ConsumedWater:  &v,
	}
}

func MapWateringPlanStatus(wateringPlanStatus sqlc.WateringPlanStatus) shared.WateringPlanStatus {
	return shared.WateringPlanStatus(wateringPlanStatus)
}
