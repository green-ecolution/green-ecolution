package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/evaluation"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func EvaluationFromResponse(source *evaluation.Evaluation) *entities.EvaluationResponse {
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

func vehicleEvaluationToResponse(source *evaluation.VehicleEvaluation) *entities.VehicleEvaluationResponse {
	if source == nil {
		return nil
	}
	return &entities.VehicleEvaluationResponse{
		NumberPlate:       source.NumberPlate,
		WateringPlanCount: source.WateringPlanCount,
	}
}

func regionEvaluationToResponse(source *evaluation.RegionEvaluation) *entities.RegionEvaluationResponse {
	if source == nil {
		return nil
	}
	return &entities.RegionEvaluationResponse{
		Name:              source.Name,
		WateringPlanCount: source.WateringPlanCount,
	}
}
