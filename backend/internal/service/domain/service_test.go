package domain

import (
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/storage/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
	"github.com/stretchr/testify/assert"
)

func TestNewService(t *testing.T) {
	t.Run("should initialize service with all repoistories", func(t *testing.T) {
		mockConfig := &config.Config{}
		mockClusterRepo := storageMock.NewMockTreeClusterRepository(t)
		mockTreeRepo := storageMock.NewMockTreeRepository(t)
		mockRegionRepo := storageMock.NewMockRegionRepository(t)
		mockInfoRepo := storageMock.NewMockInfoRepository(t)
		mockSensorRepo := storageMock.NewMockSensorRepository(t)
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		mockUserRepo := storageMock.NewMockUserRepository(t)
		mockVehicleRepo := storageMock.NewMockVehicleRepository(t)

		mockRepos := &storage.Repository{
			Auth:        mockAuthRepo,
			Info:        mockInfoRepo,
			Sensor:      mockSensorRepo,
			Tree:        mockTreeRepo,
			User:        mockUserRepo,
			TreeCluster: mockClusterRepo,
			Region:      mockRegionRepo,
			Vehicle:     mockVehicleRepo,
		}

		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree, entities.EventTypeUpdateTreeCluster, entities.EventTypeUpdateWateringPlan)
		svc := NewService(mockConfig, mockRepos, eventManager)

		assert.NotNil(t, svc)
		assert.IsType(t, &service.Services{}, svc)
		assert.NotNil(t, svc.InfoService)
		assert.NotNil(t, svc.TreeService)
		assert.NotNil(t, svc.AuthService)
		assert.NotNil(t, svc.RegionService)
		assert.NotNil(t, svc.TreeClusterService)
		assert.NotNil(t, svc.SensorService)
		assert.NotNil(t, svc.VehicleService)
	})
}
