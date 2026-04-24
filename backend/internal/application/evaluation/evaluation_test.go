package evaluation

import (
	"context"
	"testing"

	"github.com/pkg/errors"
	"github.com/stretchr/testify/assert"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/evaluation"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
)

func TestEvaluationService_GetAll(t *testing.T) {
	expectedVehicleEvaluaton := []*evaluation.VehicleEvaluation{
		{
			NumberPlate:       "B-1001",
			WateringPlanCount: int64(3),
		},
		{
			NumberPlate:       "B-1002",
			WateringPlanCount: int64(1),
		},
	}

	expectedRegionEvaluation := []*evaluation.RegionEvaluation{
		{
			Name:              "Mürwik",
			WateringPlanCount: int64(3),
		},
		{
			Name:              "Nordstadt",
			WateringPlanCount: int64(1),
		},
	}

	expectedEvaluation := &evaluation.Evaluation{
		TreeCount:             int64(10),
		TreeClusterCount:      int64(3),
		WateringPlanCount:     int64(3),
		SensorCount:           int64(2),
		TotalWaterConsumption: int64(10000),
		UserWateringPlanCount: int64(6),
		VehicleEvaluation:     expectedVehicleEvaluaton,
		RegionEvaluation:      expectedRegionEvaluation,
	}

	t.Run("should return evaluation values when successful", func(t *testing.T) {
		evaluationRepo := storageMock.NewMockEvaluationRepository(t)
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)

		svc := NewEvaluationService(evaluationRepo, clusterRepo, treeRepo, sensorRepo, wateringPlanRepo)

		clusterRepo.EXPECT().GetCount(context.Background(), cluster.TreeClusterQuery{}).Return(expectedEvaluation.TreeClusterCount, nil)
		treeRepo.EXPECT().GetCount(context.Background(), tree.TreeQuery{}).Return(expectedEvaluation.TreeCount, nil)
		sensorRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(expectedEvaluation.SensorCount, nil)
		wateringPlanRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(expectedEvaluation.WateringPlanCount, nil)
		evaluationRepo.EXPECT().GetTotalConsumedWater(context.Background()).Return(expectedEvaluation.TotalWaterConsumption, nil)
		evaluationRepo.EXPECT().GetWateringPlanUserCount(context.Background()).Return(expectedEvaluation.UserWateringPlanCount, nil)
		evaluationRepo.EXPECT().GetVehiclesWithWateringPlanCount(context.Background()).Return(expectedVehicleEvaluaton, nil)
		evaluationRepo.EXPECT().GetRegionsWithWateringPlanCount(context.Background()).Return(expectedRegionEvaluation, nil)

		evaluation, err := svc.GetEvaluation(context.Background())

		assert.NoError(t, err)
		assert.Equal(t, expectedEvaluation, evaluation)
	})

	t.Run("should return error when getting cluster count fails", func(t *testing.T) {
		evaluationRepo := storageMock.NewMockEvaluationRepository(t)
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)

		svc := NewEvaluationService(evaluationRepo, clusterRepo, treeRepo, sensorRepo, wateringPlanRepo)

		clusterRepo.EXPECT().GetCount(context.Background(), cluster.TreeClusterQuery{}).Return(int64(0), errors.New("internal error"))
		evaluation, err := svc.GetEvaluation(context.Background())

		assert.Error(t, err)
		assert.Equal(t, int64(0), evaluation.SensorCount)
		assert.Equal(t, int64(0), evaluation.TreeClusterCount)
		assert.Equal(t, int64(0), evaluation.TreeCount)
		assert.Equal(t, int64(0), evaluation.WateringPlanCount)
		assert.Equal(t, int64(0), evaluation.TotalWaterConsumption)
		assert.Equal(t, int64(0), evaluation.UserWateringPlanCount)
		assert.Empty(t, evaluation.VehicleEvaluation)
		assert.Empty(t, evaluation.RegionEvaluation)
	})

	t.Run("should return error when getting tree count fails", func(t *testing.T) {
		evaluationRepo := storageMock.NewMockEvaluationRepository(t)
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)

		svc := NewEvaluationService(evaluationRepo, clusterRepo, treeRepo, sensorRepo, wateringPlanRepo)

		clusterRepo.EXPECT().GetCount(context.Background(), cluster.TreeClusterQuery{}).Return(expectedEvaluation.TreeClusterCount, nil)
		treeRepo.EXPECT().GetCount(context.Background(), tree.TreeQuery{}).Return(int64(0), errors.New("internal error"))
		evaluation, err := svc.GetEvaluation(context.Background())

		assert.Error(t, err)
		assert.Equal(t, int64(0), evaluation.SensorCount)
		assert.Equal(t, int64(0), evaluation.TreeClusterCount)
		assert.Equal(t, int64(0), evaluation.TreeCount)
		assert.Equal(t, int64(0), evaluation.WateringPlanCount)
		assert.Equal(t, int64(0), evaluation.TotalWaterConsumption)
		assert.Equal(t, int64(0), evaluation.UserWateringPlanCount)
		assert.Empty(t, evaluation.VehicleEvaluation)
		assert.Empty(t, evaluation.RegionEvaluation)
	})

	t.Run("should return error when getting sensor count fails", func(t *testing.T) {
		evaluationRepo := storageMock.NewMockEvaluationRepository(t)
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)

		svc := NewEvaluationService(evaluationRepo, clusterRepo, treeRepo, sensorRepo, wateringPlanRepo)

		clusterRepo.EXPECT().GetCount(context.Background(), cluster.TreeClusterQuery{}).Return(expectedEvaluation.TreeClusterCount, nil)
		treeRepo.EXPECT().GetCount(context.Background(), tree.TreeQuery{}).Return(expectedEvaluation.TreeCount, nil)
		sensorRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(int64(0), errors.New("internal error"))
		evaluation, err := svc.GetEvaluation(context.Background())

		assert.Error(t, err)
		assert.Equal(t, int64(0), evaluation.SensorCount)
		assert.Equal(t, int64(0), evaluation.TreeClusterCount)
		assert.Equal(t, int64(0), evaluation.TreeCount)
		assert.Equal(t, int64(0), evaluation.WateringPlanCount)
		assert.Equal(t, int64(0), evaluation.TotalWaterConsumption)
		assert.Equal(t, int64(0), evaluation.UserWateringPlanCount)
		assert.Empty(t, evaluation.VehicleEvaluation)
		assert.Empty(t, evaluation.RegionEvaluation)
	})

	t.Run("should return error when getting watering plans count fails", func(t *testing.T) {
		evaluationRepo := storageMock.NewMockEvaluationRepository(t)
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)

		svc := NewEvaluationService(evaluationRepo, clusterRepo, treeRepo, sensorRepo, wateringPlanRepo)

		clusterRepo.EXPECT().GetCount(context.Background(), cluster.TreeClusterQuery{}).Return(expectedEvaluation.TreeClusterCount, nil)
		treeRepo.EXPECT().GetCount(context.Background(), tree.TreeQuery{}).Return(expectedEvaluation.TreeCount, nil)
		sensorRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(expectedEvaluation.SensorCount, nil)
		wateringPlanRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(int64(0), errors.New("internal error"))

		evaluation, err := svc.GetEvaluation(context.Background())

		assert.Error(t, err)
		assert.Equal(t, int64(0), evaluation.SensorCount)
		assert.Equal(t, int64(0), evaluation.TreeClusterCount)
		assert.Equal(t, int64(0), evaluation.TreeCount)
		assert.Equal(t, int64(0), evaluation.WateringPlanCount)
		assert.Equal(t, int64(0), evaluation.TotalWaterConsumption)
		assert.Equal(t, int64(0), evaluation.UserWateringPlanCount)
		assert.Empty(t, evaluation.VehicleEvaluation)
		assert.Empty(t, evaluation.RegionEvaluation)
	})

	t.Run("should return error when getting total water consumption fails", func(t *testing.T) {
		evaluationRepo := storageMock.NewMockEvaluationRepository(t)
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)

		svc := NewEvaluationService(evaluationRepo, clusterRepo, treeRepo, sensorRepo, wateringPlanRepo)

		clusterRepo.EXPECT().GetCount(context.Background(), cluster.TreeClusterQuery{}).Return(expectedEvaluation.TreeClusterCount, nil)
		treeRepo.EXPECT().GetCount(context.Background(), tree.TreeQuery{}).Return(expectedEvaluation.TreeCount, nil)
		sensorRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(expectedEvaluation.SensorCount, nil)
		wateringPlanRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(expectedEvaluation.WateringPlanCount, nil)
		evaluationRepo.EXPECT().GetTotalConsumedWater(context.Background()).Return(int64(0), errors.New("internal error"))

		evaluation, err := svc.GetEvaluation(context.Background())

		assert.Error(t, err)
		assert.Equal(t, int64(0), evaluation.SensorCount)
		assert.Equal(t, int64(0), evaluation.TreeClusterCount)
		assert.Equal(t, int64(0), evaluation.TreeCount)
		assert.Equal(t, int64(0), evaluation.WateringPlanCount)
		assert.Equal(t, int64(0), evaluation.TotalWaterConsumption)
		assert.Equal(t, int64(0), evaluation.UserWateringPlanCount)
		assert.Empty(t, evaluation.VehicleEvaluation)
		assert.Empty(t, evaluation.RegionEvaluation)
	})

	t.Run("should return error when getting all linked user count to a watering plan fails", func(t *testing.T) {
		evaluationRepo := storageMock.NewMockEvaluationRepository(t)
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)

		svc := NewEvaluationService(evaluationRepo, clusterRepo, treeRepo, sensorRepo, wateringPlanRepo)

		clusterRepo.EXPECT().GetCount(context.Background(), cluster.TreeClusterQuery{}).Return(expectedEvaluation.TreeClusterCount, nil)
		treeRepo.EXPECT().GetCount(context.Background(), tree.TreeQuery{}).Return(expectedEvaluation.TreeCount, nil)
		sensorRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(expectedEvaluation.SensorCount, nil)
		wateringPlanRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(expectedEvaluation.WateringPlanCount, nil)
		evaluationRepo.EXPECT().GetTotalConsumedWater(context.Background()).Return(expectedEvaluation.TotalWaterConsumption, nil)
		evaluationRepo.EXPECT().GetWateringPlanUserCount(context.Background()).Return(int64(0), errors.New("internal error"))

		evaluation, err := svc.GetEvaluation(context.Background())

		assert.Error(t, err)
		assert.Equal(t, int64(0), evaluation.SensorCount)
		assert.Equal(t, int64(0), evaluation.TreeClusterCount)
		assert.Equal(t, int64(0), evaluation.TreeCount)
		assert.Equal(t, int64(0), evaluation.WateringPlanCount)
		assert.Equal(t, int64(0), evaluation.TotalWaterConsumption)
		assert.Equal(t, int64(0), evaluation.UserWateringPlanCount)
		assert.Empty(t, evaluation.VehicleEvaluation)
		assert.Empty(t, evaluation.RegionEvaluation)
	})

	t.Run("should return error when getting vehicle evaluation fails", func(t *testing.T) {
		evaluationRepo := storageMock.NewMockEvaluationRepository(t)
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)

		svc := NewEvaluationService(evaluationRepo, clusterRepo, treeRepo, sensorRepo, wateringPlanRepo)

		clusterRepo.EXPECT().GetCount(context.Background(), cluster.TreeClusterQuery{}).Return(expectedEvaluation.TreeClusterCount, nil)
		treeRepo.EXPECT().GetCount(context.Background(), tree.TreeQuery{}).Return(expectedEvaluation.TreeCount, nil)
		sensorRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(expectedEvaluation.SensorCount, nil)
		wateringPlanRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(expectedEvaluation.WateringPlanCount, nil)
		evaluationRepo.EXPECT().GetTotalConsumedWater(context.Background()).Return(expectedEvaluation.TotalWaterConsumption, nil)
		evaluationRepo.EXPECT().GetWateringPlanUserCount(context.Background()).Return(expectedEvaluation.UserWateringPlanCount, nil)
		evaluationRepo.EXPECT().GetVehiclesWithWateringPlanCount(context.Background()).Return(nil, errors.New("internal error"))

		evaluation, err := svc.GetEvaluation(context.Background())

		assert.Error(t, err)
		assert.Equal(t, int64(0), evaluation.SensorCount)
		assert.Equal(t, int64(0), evaluation.TreeClusterCount)
		assert.Equal(t, int64(0), evaluation.TreeCount)
		assert.Equal(t, int64(0), evaluation.WateringPlanCount)
		assert.Equal(t, int64(0), evaluation.TotalWaterConsumption)
		assert.Equal(t, int64(0), evaluation.UserWateringPlanCount)
		assert.Empty(t, evaluation.VehicleEvaluation)
		assert.Empty(t, evaluation.RegionEvaluation)
	})

	t.Run("should return error when getting region evaluation fails", func(t *testing.T) {
		evaluationRepo := storageMock.NewMockEvaluationRepository(t)
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)

		svc := NewEvaluationService(evaluationRepo, clusterRepo, treeRepo, sensorRepo, wateringPlanRepo)

		clusterRepo.EXPECT().GetCount(context.Background(), cluster.TreeClusterQuery{}).Return(expectedEvaluation.TreeClusterCount, nil)
		treeRepo.EXPECT().GetCount(context.Background(), tree.TreeQuery{}).Return(expectedEvaluation.TreeCount, nil)
		sensorRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(expectedEvaluation.SensorCount, nil)
		wateringPlanRepo.EXPECT().GetCount(context.Background(), shared.Query{}).Return(expectedEvaluation.WateringPlanCount, nil)
		evaluationRepo.EXPECT().GetTotalConsumedWater(context.Background()).Return(expectedEvaluation.TotalWaterConsumption, nil)
		evaluationRepo.EXPECT().GetWateringPlanUserCount(context.Background()).Return(expectedEvaluation.UserWateringPlanCount, nil)
		evaluationRepo.EXPECT().GetVehiclesWithWateringPlanCount(context.Background()).Return(expectedVehicleEvaluaton, nil)
		evaluationRepo.EXPECT().GetRegionsWithWateringPlanCount(context.Background()).Return(nil, errors.New("internal error"))

		evaluation, err := svc.GetEvaluation(context.Background())

		assert.Error(t, err)
		assert.Equal(t, int64(0), evaluation.SensorCount)
		assert.Equal(t, int64(0), evaluation.TreeClusterCount)
		assert.Equal(t, int64(0), evaluation.TreeCount)
		assert.Equal(t, int64(0), evaluation.WateringPlanCount)
		assert.Equal(t, int64(0), evaluation.UserWateringPlanCount)
		assert.Equal(t, int64(0), evaluation.TotalWaterConsumption)
		assert.Empty(t, evaluation.VehicleEvaluation)
		assert.Empty(t, evaluation.RegionEvaluation)
	})
}
