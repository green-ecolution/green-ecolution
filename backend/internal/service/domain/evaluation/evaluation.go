package evaluation

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
)

type EvaluationService struct {
	treeClusterRepo  storage.TreeClusterRepository
	treeRepo         storage.TreeRepository
	sensorRepo       storage.SensorRepository
	wateringPlanRepo storage.WateringPlanRepository
	vehicleRepo      storage.VehicleRepository
}

func NewEvaluationService(
	treeClusterRepo storage.TreeClusterRepository,
	treeRepo storage.TreeRepository,
	sensorRepo storage.SensorRepository,
	wateringPlanRepo storage.WateringPlanRepository,
	vehicleRepo storage.VehicleRepository,
) service.EvaluationService {
	return &EvaluationService{
		treeClusterRepo:  treeClusterRepo,
		treeRepo:         treeRepo,
		sensorRepo:       sensorRepo,
		wateringPlanRepo: wateringPlanRepo,
		vehicleRepo:      vehicleRepo,
	}
}

func (e *EvaluationService) GetEvaluation(ctx context.Context) (*entities.Evaluation, error) {
	log := logger.GetLogger(ctx)

	clusterCount, err := e.treeClusterRepo.GetCount(ctx, entities.TreeClusterQuery{})
	if err != nil {
		log.Error("failed to get treecluster count", "error", err)
		return &entities.Evaluation{}, err
	}

	treeCount, err := e.treeRepo.GetCount(ctx, entities.TreeQuery{})
	if err != nil {
		log.Error("failed to get tree count", "error", err)
		return &entities.Evaluation{}, err
	}

	sensorCount, err := e.sensorRepo.GetCount(ctx, entities.Query{})
	if err != nil {
		log.Error("failed to get sensor count", "error", err)
		return &entities.Evaluation{}, err
	}

	wateringPlanCount, err := e.wateringPlanRepo.GetCount(ctx, entities.Query{})
	if err != nil {
		log.Error("failed to get sensor count", "error", err)
		return &entities.Evaluation{}, err
	}

	totalConsumedWater, err := e.wateringPlanRepo.GetTotalConsumedWater(ctx)
	if err != nil {
		log.Error("failed to get sensor count", "error", err)
		return &entities.Evaluation{}, err
	}

	userCount, err := e.wateringPlanRepo.GetAllUserCount(ctx)
	if err != nil {
		log.Error("failed to get user count linked to watering plans", "error", err)
		return &entities.Evaluation{}, err
	}

	vehicleEvaluation, err := e.vehicleRepo.GetAllWithWateringPlanCount(ctx)
	if err != nil {
		log.Error("failed to get vehicle evaluation", "error", err)
		return &entities.Evaluation{}, err
	}

	regionEvaluation, err := e.treeClusterRepo.GetAllRegionsWithWateringPlanCount(ctx)
	if err != nil {
		log.Error("failed to get region evaluation", "error", err)
		return &entities.Evaluation{}, err
	}

	evaluation := &entities.Evaluation{
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
