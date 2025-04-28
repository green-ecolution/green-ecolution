package service

import (
	"testing"

	serviceMock "github.com/green-ecolution/backend/internal/service/_mock"
	"github.com/stretchr/testify/assert"
)

func TestAllServiceReady(t *testing.T) {
	t.Run("should return true if all service implemented the ServiceReady interface", func(t *testing.T) {
		// given
		infoSvc := serviceMock.NewMockInfoService(t)
		treeSvc := serviceMock.NewMockTreeService(t)
		authSvc := serviceMock.NewMockAuthService(t)
		regionSvc := serviceMock.NewMockRegionService(t)
		treeClusterSvc := serviceMock.NewMockTreeClusterService(t)
		sensorSvc := serviceMock.NewMockSensorService(t)
		vehicleSvc := serviceMock.NewMockVehicleService(t)
		pluginSvc := serviceMock.NewMockPluginService(t)
		wateringPlanSvc := serviceMock.NewMockWateringPlanService(t)
		evaluationSvc := serviceMock.NewMockEvaluationService(t)
		svc := Services{
			InfoService:         infoSvc,
			TreeService:         treeSvc,
			AuthService:         authSvc,
			RegionService:       regionSvc,
			TreeClusterService:  treeClusterSvc,
			SensorService:       sensorSvc,
			VehicleService:      vehicleSvc,
			PluginService:       pluginSvc,
			WateringPlanService: wateringPlanSvc,
			EvaluationService:   evaluationSvc,
		}

		// when
		infoSvc.EXPECT().Ready().Return(true)
		treeSvc.EXPECT().Ready().Return(true)
		authSvc.EXPECT().Ready().Return(true)
		regionSvc.EXPECT().Ready().Return(true)
		treeClusterSvc.EXPECT().Ready().Return(true)
		sensorSvc.EXPECT().Ready().Return(true)
		vehicleSvc.EXPECT().Ready().Return(true)
		pluginSvc.EXPECT().Ready().Return(true)
		wateringPlanSvc.EXPECT().Ready().Return(true)
		evaluationSvc.EXPECT().Ready().Return(true)

		ready := svc.AllServicesReady()

		// then
		assert.True(t, ready)
	})
}
