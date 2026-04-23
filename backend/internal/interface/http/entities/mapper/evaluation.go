package mapper

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func EvaluationFromResponse(source *domain.Evaluation) *entities.EvaluationResponse {
	if source == nil {
		return nil
	}
	return &entities.EvaluationResponse{
		TreeCount:             source.TreeCount,
		TreeClusterCount:      source.TreeClusterCount,
		SensorCount:           source.SensorCount,
		WateringPlanCount:     source.WateringPlanCount,
		TotalWaterConsumption: source.TotalWaterConsumption,
		UserWateringPlanCount: source.UserWateringPlanCount,
		VehicleEvaluation:     utils.MapSlice(source.VehicleEvaluation, vehicleEvaluationToResponse),
		RegionEvaluation:      utils.MapSlice(source.RegionEvaluation, regionEvaluationToResponse),
	}
}

func vehicleEvaluationToResponse(source *domain.VehicleEvaluation) *entities.VehicleEvaluationResponse {
	if source == nil {
		return nil
	}
	return &entities.VehicleEvaluationResponse{
		NumberPlate:       source.NumberPlate,
		WateringPlanCount: source.WateringPlanCount,
	}
}

func regionEvaluationToResponse(source *domain.RegionEvaluation) *entities.RegionEvaluationResponse {
	if source == nil {
		return nil
	}
	return &entities.RegionEvaluationResponse{
		Name:              source.Name,
		WateringPlanCount: source.WateringPlanCount,
	}
}
