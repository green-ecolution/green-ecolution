package evaluation

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

type EvaluationService struct {
	treeClusterRepo  shared.TreeClusterRepository
	treeRepo         shared.TreeRepository
	sensorRepo       shared.SensorRepository
	wateringPlanRepo shared.WateringPlanRepository
	vehicleRepo      shared.VehicleRepository
}

func NewEvaluationService(
	treeClusterRepo shared.TreeClusterRepository,
	treeRepo shared.TreeRepository,
	sensorRepo shared.SensorRepository,
	wateringPlanRepo shared.WateringPlanRepository,
	vehicleRepo shared.VehicleRepository,
) ports.EvaluationService {
	return &EvaluationService{
		treeClusterRepo:  treeClusterRepo,
		treeRepo:         treeRepo,
		sensorRepo:       sensorRepo,
		wateringPlanRepo: wateringPlanRepo,
		vehicleRepo:      vehicleRepo,
	}
}

func (e *EvaluationService) GetEvaluation(ctx context.Context) (*shared.Evaluation, error) {
	log := logger.GetLogger(ctx)

	clusterCount, err := e.treeClusterRepo.GetCount(ctx, shared.TreeClusterQuery{})
	if err != nil {
		log.Error("failed to get treecluster count", "error", err)
		return &shared.Evaluation{}, err
	}

	treeCount, err := e.treeRepo.GetCount(ctx, shared.TreeQuery{})
	if err != nil {
		log.Error("failed to get tree count", "error", err)
		return &shared.Evaluation{}, err
	}

	sensorCount, err := e.sensorRepo.GetCount(ctx, shared.Query{})
	if err != nil {
		log.Error("failed to get sensor count", "error", err)
		return &shared.Evaluation{}, err
	}

	wateringPlanCount, err := e.wateringPlanRepo.GetCount(ctx, shared.Query{})
	if err != nil {
		log.Error("failed to get sensor count", "error", err)
		return &shared.Evaluation{}, err
	}

	totalConsumedWater, err := e.wateringPlanRepo.GetTotalConsumedWater(ctx)
	if err != nil {
		log.Error("failed to get sensor count", "error", err)
		return &shared.Evaluation{}, err
	}

	userCount, err := e.wateringPlanRepo.GetAllUserCount(ctx)
	if err != nil {
		log.Error("failed to get user count linked to watering plans", "error", err)
		return &shared.Evaluation{}, err
	}

	vehicleEvaluation, err := e.vehicleRepo.GetAllWithWateringPlanCount(ctx)
	if err != nil {
		log.Error("failed to get vehicle evaluation", "error", err)
		return &shared.Evaluation{}, err
	}

	regionEvaluation, err := e.treeClusterRepo.GetAllRegionsWithWateringPlanCount(ctx)
	if err != nil {
		log.Error("failed to get region evaluation", "error", err)
		return &shared.Evaluation{}, err
	}

	evaluation := &shared.Evaluation{
		TreeClusterCount:      clusterCount,
		TreeCount:             treeCount,
		SensorCount:           sensorCount,
		WateringPlanCount:     wateringPlanCount,
		TotalWaterConsumption: totalConsumedWater,
		UserWateringPlanCount: userCount,
		VehicleEvaluation:     vehicleEvaluation,
		RegionEvaluation:      regionEvaluation,
	}

	return evaluation, nil
}

func (e *EvaluationService) Ready() bool {
	return e.treeClusterRepo != nil &&
		e.treeRepo != nil &&
		e.sensorRepo != nil &&
		e.wateringPlanRepo != nil &&
		e.vehicleRepo != nil
}
