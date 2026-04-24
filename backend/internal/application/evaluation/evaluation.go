package evaluation

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/evaluation"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/watering"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

type EvaluationService struct {
	evaluationRepo   evaluation.EvaluationRepository
	treeClusterRepo  cluster.TreeClusterRepository
	treeRepo         tree.TreeRepository
	sensorRepo       sensor.SensorRepository
	wateringPlanRepo watering.WateringPlanRepository
}

func NewEvaluationService(
	evaluationRepo evaluation.EvaluationRepository,
	treeClusterRepo cluster.TreeClusterRepository,
	treeRepo tree.TreeRepository,
	sensorRepo sensor.SensorRepository,
	wateringPlanRepo watering.WateringPlanRepository,
) ports.EvaluationService {
	return &EvaluationService{
		evaluationRepo:   evaluationRepo,
		treeClusterRepo:  treeClusterRepo,
		treeRepo:         treeRepo,
		sensorRepo:       sensorRepo,
		wateringPlanRepo: wateringPlanRepo,
	}
}

func (e *EvaluationService) GetEvaluation(ctx context.Context) (*evaluation.Evaluation, error) {
	log := logger.GetLogger(ctx)

	clusterCount, err := e.treeClusterRepo.GetCount(ctx, cluster.TreeClusterQuery{})
	if err != nil {
		log.Error("failed to get treecluster count", "error", err)
		return &evaluation.Evaluation{}, err
	}

	treeCount, err := e.treeRepo.GetCount(ctx, tree.TreeQuery{})
	if err != nil {
		log.Error("failed to get tree count", "error", err)
		return &evaluation.Evaluation{}, err
	}

	sensorCount, err := e.sensorRepo.GetCount(ctx, shared.Query{})
	if err != nil {
		log.Error("failed to get sensor count", "error", err)
		return &evaluation.Evaluation{}, err
	}

	wateringPlanCount, err := e.wateringPlanRepo.GetCount(ctx, shared.Query{})
	if err != nil {
		log.Error("failed to get watering plan count", "error", err)
		return &evaluation.Evaluation{}, err
	}

	totalConsumedWater, err := e.evaluationRepo.GetTotalConsumedWater(ctx)
	if err != nil {
		log.Error("failed to get total consumed water", "error", err)
		return &evaluation.Evaluation{}, err
	}

	userCount, err := e.evaluationRepo.GetWateringPlanUserCount(ctx)
	if err != nil {
		log.Error("failed to get user count linked to watering plans", "error", err)
		return &evaluation.Evaluation{}, err
	}

	vehicleEvaluation, err := e.evaluationRepo.GetVehiclesWithWateringPlanCount(ctx)
	if err != nil {
		log.Error("failed to get vehicle evaluation", "error", err)
		return &evaluation.Evaluation{}, err
	}

	regionEvaluation, err := e.evaluationRepo.GetRegionsWithWateringPlanCount(ctx)
	if err != nil {
		log.Error("failed to get region evaluation", "error", err)
		return &evaluation.Evaluation{}, err
	}

	eval := &evaluation.Evaluation{
		TreeClusterCount:      clusterCount,
		TreeCount:             treeCount,
		SensorCount:           sensorCount,
		WateringPlanCount:     wateringPlanCount,
		TotalWaterConsumption: totalConsumedWater,
		UserWateringPlanCount: userCount,
		VehicleEvaluation:     vehicleEvaluation,
		RegionEvaluation:      regionEvaluation,
	}

	return eval, nil
}

func (e *EvaluationService) Ready() bool {
	return e.evaluationRepo != nil &&
		e.treeClusterRepo != nil &&
		e.treeRepo != nil &&
		e.sensorRepo != nil &&
		e.wateringPlanRepo != nil
}
