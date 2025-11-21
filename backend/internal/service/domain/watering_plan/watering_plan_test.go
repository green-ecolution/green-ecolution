package wateringplan

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/storage/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

var globalEventManager = worker.NewEventManager() //entities.EventTypeUpdateWateringPlan

func TestWateringPlanService_GetAll(t *testing.T) {
	ctx := context.Background()

	t.Run("should return all watering plans when successful", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetAll(ctx, entities.Query{}).Return(allTestWateringPlans, int64(len(allTestWateringPlans)), nil)

		// when
		wateringPlans, totalCount, err := svc.GetAll(ctx, entities.Query{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, allTestWateringPlans, wateringPlans)
		assert.Equal(t, totalCount, int64(len(allTestWateringPlans)))
	})

	t.Run("should return all watering plans when successful with provider", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetAll(ctx, entities.Query{Provider: "test-provider"}).Return(allTestWateringPlans, int64(len(allTestWateringPlans)), nil)

		// when
		wateringPlans, totalCount, err := svc.GetAll(ctx, entities.Query{Provider: "test-provider"})

		// then
		assert.NoError(t, err)
		assert.Equal(t, allTestWateringPlans, wateringPlans)
		assert.Equal(t, totalCount, int64(len(allTestWateringPlans)))
	})

	t.Run("should return empty slice when no watering plans are found", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetAll(ctx, entities.Query{}).Return([]*entities.WateringPlan{}, int64(0), nil)

		// when
		wateringPlans, totalCount, err := svc.GetAll(ctx, entities.Query{})

		// then
		assert.NoError(t, err)
		assert.Empty(t, wateringPlans)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error when GetAll fails", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		expectedErr := errors.New("GetAll failed")
		wateringPlanRepo.EXPECT().GetAll(ctx, entities.Query{}).Return(nil, int64(0), expectedErr)

		// when
		wateringPlans, totalCount, err := svc.GetAll(ctx, entities.Query{})

		// then
		assert.Error(t, err)
		assert.Nil(t, wateringPlans)
		assert.Equal(t, totalCount, int64(0))
		// assert.Equal(t, "500: GetAll failed", err.Error())
	})
}

func TestWateringPlanService_GetByID(t *testing.T) {
	ctx := context.Background()

	t.Run("should return watering plan when found", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		id := int32(1)
		expectedPlan := allTestWateringPlans[0]
		wateringPlanRepo.EXPECT().GetByID(ctx, id).Return(expectedPlan, nil)

		// when
		wateringPlan, err := svc.GetByID(ctx, id)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedPlan, wateringPlan)
	})

	t.Run("should return error if watering plan not found", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		id := int32(1)
		expectedErr := storage.ErrEntityNotFound("not found")
		wateringPlanRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedErr)

		// when
		wateringPlan, err := svc.GetByID(ctx, id)

		// then
		assert.Error(t, err)
		assert.Nil(t, wateringPlan)
		// assert.Equal(t, "404: watering plan not found", err.Error())
	})
}

func TestWateringPlanService_Create(t *testing.T) {
	ctx := context.Background()
	testUUIDString := "6a1078e8-80fd-458f-b74e-e388fe2dd6ab"
	testUUID, err := uuid.Parse(testUUIDString)
	if err != nil {
		t.Fatal(err)
	}

	newWateringPlan := &entities.WateringPlanCreate{
		Date:           time.Date(2024, 9, 26, 0, 0, 0, 0, time.UTC),
		Description:    "New watering plan",
		TransporterID:  utils.P(int32(2)),
		TrailerID:      utils.P(int32(1)),
		TreeClusterIDs: []*int32{utils.P(int32(1)), utils.P(int32(2))},
		UserIDs:        []*uuid.UUID{&testUUID},
	}

	t.Run("should successfully create a new watering plan", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserTbz}, nil)

		wateringPlanRepo.EXPECT().Create(
			ctx,
			mock.Anything,
		).Return(allTestWateringPlans[0], nil)

		wateringPlanRepo.EXPECT().Update(
			ctx,
			allTestWateringPlans[0].ID,
			mock.Anything,
		).Return(nil)

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.NoError(t, err)
		assert.Equal(t, allTestWateringPlans[0], result)
	})

	t.Run("should successfully create a new watering plan without a trailer", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		newWateringPlan := &entities.WateringPlanCreate{
			Date:           time.Date(2024, 9, 26, 0, 0, 0, 0, time.UTC),
			Description:    "New watering plan",
			TransporterID:  utils.P(int32(2)),
			TreeClusterIDs: []*int32{utils.P(int32(1)), utils.P(int32(2))},
			UserIDs:        []*uuid.UUID{&testUUID},
		}

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserTbz}, nil)

		wateringPlanRepo.EXPECT().Create(
			ctx,
			mock.Anything,
		).Return(allTestWateringPlans[0], nil)

		wateringPlanRepo.EXPECT().Update(
			ctx,
			allTestWateringPlans[0].ID,
			mock.Anything,
		).Return(nil)

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.NoError(t, err)
		assert.Equal(t, allTestWateringPlans[0], result)
	})

	t.Run("should return an error when finding treeclusters fails", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(nil, storage.ErrConnectionClosed)

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: connection is closed")
	})

	t.Run("should return an error when treecluster are empty", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return([]*entities.TreeCluster{}, nil)

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: treecluster not found")
	})

	t.Run("should return an error when transporter is not found", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(nil, storage.ErrVehicleNotFound)

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: vehicle not found")
	})

	t.Run("should return an error when users are empty", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{}, nil)

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		//assert.EqualError(t, err, "404: user not found")
	})

	t.Run("should return an error when finding users fails", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check user
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return(nil, storage.ErrUserNotFound)

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: user not found")
	})

	t.Run("should return an error when one user has not correct user role", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserGreenEcolution}, nil)

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "400: user has an incorrect role")
	})

	t.Run("should return an error when user has no role", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{{Roles: []entities.UserRole{}}}, nil)

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "400: user has an incorrect role")
	})

	t.Run("should return an error when driving licenses are not matching", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserCar}, nil)

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.ErrorContains(t, err, "400")
		// assert.ErrorContains(t, err, "does not have the required license")
	})

	t.Run("should return an error when creating watering plan fails", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		expectedErr := errors.New("Failed to create watering plan")

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserTbz}, nil)

		wateringPlanRepo.EXPECT().Create(
			ctx,
			mock.Anything,
		).Return(nil, expectedErr)

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: Failed to create watering plan")
	})

	t.Run("should return validation error when TreeClusterIDs contains nil pointers", func(t *testing.T) {
		// given
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		newWateringPlan := &entities.WateringPlanCreate{
			Date:           time.Date(2024, 9, 26, 0, 0, 0, 0, time.UTC),
			Description:    "New watering plan with nil TreeClusterIDs",
			TransporterID:  utils.P(int32(2)),
			TreeClusterIDs: []*int32{nil, nil},
			UserIDs:        []*uuid.UUID{&testUUID},
		}

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error on empty date", func(t *testing.T) {
		// given
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		newWateringPlan.Date = time.Time{}

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error on empty transporter", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		newWateringPlan := &entities.WateringPlanCreate{
			Date:        time.Date(2024, 9, 26, 0, 0, 0, 0, time.UTC),
			Description: "New watering plan",
			UserIDs:     []*uuid.UUID{&testUUID},
		}

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error on empty treeclusters", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		newWateringPlan := &entities.WateringPlanCreate{
			Date:          time.Date(2024, 9, 26, 0, 0, 0, 0, time.UTC),
			Description:   "New watering plan",
			TransporterID: utils.P(int32(2)),
			UserIDs:       []*uuid.UUID{&testUUID},
		}

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error on empty users", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		newWateringPlan := &entities.WateringPlanCreate{
			Date:          time.Date(2024, 9, 26, 0, 0, 0, 0, time.UTC),
			Description:   "New watering plan",
			TransporterID: utils.P(int32(2)),
		}

		// when
		result, err := svc.Create(ctx, newWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})
}

func TestWateringPlanService_Update(t *testing.T) {
	ctx := context.Background()
	testUUIDString := "6a1078e8-80fd-458f-b74e-e388fe2dd6ab"
	testUUID, err := uuid.Parse(testUUIDString)
	if err != nil {
		t.Fatal(err)
	}

	updatedWateringPlan := &entities.WateringPlanUpdate{
		Date:             time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
		Description:      "New watering plan for the east side of the city",
		TransporterID:    utils.P(int32(2)),
		TrailerID:        utils.P(int32(1)),
		TreeClusterIDs:   []*int32{utils.P(int32(1)), utils.P(int32(2))},
		UserIDs:          []*uuid.UUID{&testUUID},
		Status:           entities.WateringPlanStatusActive,
		CancellationNote: "",
	}

	t.Run("should successfully update a watering plan", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserTbz}, nil)

		wateringPlanRepo.EXPECT().Update(
			ctx,
			int32(1),
			mock.Anything,
		).Return(nil)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.NoError(t, err)
		assert.Equal(t, allTestWateringPlans[0], result)
	})

	t.Run("should successfully update a watering plan with evaluation", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		updatedWateringPlan := &entities.WateringPlanUpdate{
			Date:             time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
			Status:           entities.WateringPlanStatusFinished,
			CancellationNote: "",
			Description:      "New watering plan for the east side of the city",
			TransporterID:    utils.P(int32(2)),
			TreeClusterIDs:   []*int32{utils.P(int32(1)), utils.P(int32(2))},
			UserIDs:          []*uuid.UUID{&testUUID},
			Evaluation: []*entities.EvaluationValue{
				{
					WateringPlanID: int32(3),
					TreeClusterID:  1,
					ConsumedWater:  utils.P(100.00),
				},
			},
		}

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(3),
		).Return(allTestWateringPlans[2], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserTbz}, nil)

		wateringPlanRepo.EXPECT().Update(
			ctx,
			int32(3),
			mock.Anything,
		).Return(nil)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(3),
		).Return(allTestWateringPlans[2], nil)

		// when
		result, err := svc.Update(ctx, int32(3), updatedWateringPlan)

		// then
		assert.NoError(t, err)
		assert.Equal(t, allTestWateringPlans[2], result)
	})

	t.Run("should successfully update a watering plan without a trailer", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		updatedWateringPlan := &entities.WateringPlanUpdate{
			Date:             time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
			Status:           entities.WateringPlanStatusActive,
			CancellationNote: "",
			Description:      "New watering plan for the east side of the city",
			TransporterID:    utils.P(int32(2)),
			TreeClusterIDs:   []*int32{utils.P(int32(1)), utils.P(int32(2))},
			UserIDs:          []*uuid.UUID{&testUUID},
		}

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserTbz}, nil)

		wateringPlanRepo.EXPECT().Update(
			ctx,
			int32(1),
			mock.Anything,
		).Return(nil)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.NoError(t, err)
		assert.Equal(t, allTestWateringPlans[0], result)
	})

	t.Run("should return an error when finding treeclusters fails", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(nil, storage.ErrConnectionClosed)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: connection is closed")
	})

	t.Run("should return an error when treecluster are empty", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return([]*entities.TreeCluster{}, nil)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: treecluster not found")
	})

	t.Run("should return an error when transporter is not found", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(nil, storage.ErrVehicleNotFound)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: vehicle not found")
	})

	t.Run("should return an error when users are empty", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{}, nil)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		//assert.EqualError(t, err, "404: user not found")
	})

	t.Run("should return an error when one user has not correct user role", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserGreenEcolution}, nil)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "400: user has an incorrect role")
	})

	t.Run("should return an error when user has no roles", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{{Roles: []entities.UserRole{}}}, nil)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "400: user has an incorrect role")
	})

	t.Run("should return an error when users is not found", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return(nil, storage.ErrUserNotFound)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: user not found")
	})

	t.Run("should return an error when driving licenses aren't matching", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserCar}, nil)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.ErrorContains(t, err, "400")
		// assert.ErrorContains(t, err, "does not have the required license")
	})

	t.Run("should return an error when watering plan does not exist", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserTbz}, nil)

		wateringPlanRepo.EXPECT().Update(
			ctx,
			int32(1),
			mock.Anything,
		).Return(storage.ErrEntityNotFound("not found"))

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: watering plan not found")
	})

	t.Run("should return an error when the update fails", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		expectedErr := errors.New("failed to update watering plan")

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		// check trailer
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestVehicles[0], nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserTbz}, nil)

		wateringPlanRepo.EXPECT().Update(
			ctx,
			int32(1),
			mock.Anything,
		).Return(expectedErr)

		// when
		result, err := svc.Update(context.Background(), int32(1), updatedWateringPlan)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to update watering plan")
	})

	t.Run("should return error if cancellation note is not empty but the status is not »canceled«", func(t *testing.T) {
		// given
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		updatedWateringPlan.Status = entities.WateringPlanStatusActive
		updatedWateringPlan.CancellationNote = "This is a note"

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "400: cancellation note can only be set if watering plan is canceled")
	})

	t.Run("should return error if the evaluation is not empty but the status is not »finished«", func(t *testing.T) {
		// given
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		updatedWateringPlan.Status = entities.WateringPlanStatusPlanned
		updatedWateringPlan.CancellationNote = ""
		updatedWateringPlan.Evaluation = []*entities.EvaluationValue{
			{
				WateringPlanID: int32(1),
				TreeClusterID:  1,
				ConsumedWater:  utils.P(100.00),
			},
		}

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(allTestWateringPlans[0], nil)

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "400: evaluation values can only be set if the watering plan has been finished")
	})

	t.Run("should return validation error when TreeClusterIDs contains nil pointers", func(t *testing.T) {
		// given
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		updatedWateringPlan := &entities.WateringPlanUpdate{
			Date:           time.Date(2024, 9, 26, 0, 0, 0, 0, time.UTC),
			Description:    "New watering plan with nil TreeClusterIDs",
			TransporterID:  utils.P(int32(2)),
			TreeClusterIDs: []*int32{nil, nil},
			UserIDs:        []*uuid.UUID{&testUUID},
		}

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error when UserIDs contains nil pointers", func(t *testing.T) {
		// given
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		updatedWateringPlan := &entities.WateringPlanUpdate{
			Date:        time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
			Status:      entities.WateringPlanStatusActive,
			Description: "New watering plan for the east side of the city",
			UserIDs:     []*uuid.UUID{nil, nil},
		}

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error on empty date", func(t *testing.T) {
		// given
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		updatedWateringPlan.Date = time.Time{}

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error on wrong status", func(t *testing.T) {
		// given
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		updatedWateringPlan.Status = "test"

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error on empty users", func(t *testing.T) {
		// given
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		updatedWateringPlan := &entities.WateringPlanUpdate{
			Date:        time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
			Status:      entities.WateringPlanStatusActive,
			Description: "New watering plan for the east side of the city",
			UserIDs:     []*uuid.UUID{&testUUID},
		}

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error on empty transporter", func(t *testing.T) {
		// given
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		updatedWateringPlan := &entities.WateringPlanUpdate{
			Date:        time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
			Status:      entities.WateringPlanStatusActive,
			Description: "New watering plan for the east side of the city",
			UserIDs:     []*uuid.UUID{&testUUID},
		}

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error on empty treeclusters", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		updatedWateringPlan := &entities.WateringPlanUpdate{
			Date:          time.Date(2024, 9, 26, 0, 0, 0, 0, time.UTC),
			Description:   "Updated watering plan",
			Status:        entities.WateringPlanStatusActive,
			TransporterID: utils.P(int32(2)),
			UserIDs:       []*uuid.UUID{&testUUID},
		}

		// when
		result, err := svc.Update(ctx, int32(1), updatedWateringPlan)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})
}

func TestWateringPlanService_EventSystem(t *testing.T) {
	t.Run("should send update watering plan event on update watering plan", func(t *testing.T) {
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		testUUIDString := "6a1078e8-80fd-458f-b74e-e388fe2dd6ab"
		testUUID, err := uuid.Parse(testUUIDString)
		if err != nil {
			t.Fatal(err)
		}

		prevWp := entities.WateringPlan{
			ID:           1,
			Date:         time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
			TreeClusters: []*entities.TreeCluster{{ID: 1}},
			Status:       entities.WateringPlanStatusActive,
			UserIDs:      []*uuid.UUID{&testUUID},
		}

		updatedWateringPlan := &entities.WateringPlanUpdate{
			Date:           time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
			TransporterID:  utils.P(int32(2)),
			TreeClusterIDs: []*int32{utils.P(int32(1)), utils.P(int32(2))},
			UserIDs:        []*uuid.UUID{&testUUID},
			Status:         entities.WateringPlanStatusActive,
		}

		expectedWp := entities.WateringPlan{
			ID:           1,
			Date:         time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
			TreeClusters: []*entities.TreeCluster{{ID: 1}},
			Status:       entities.WateringPlanStatusActive,
			UserIDs:      []*uuid.UUID{&testUUID},
		}

		// Event
		eventManager := worker.NewEventManager(entities.EventTypeUpdateWateringPlan)
		expectedEvent := entities.NewEventUpdateWateringPlan(&prevWp, &expectedWp)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		wateringPlanRepo.EXPECT().GetByID(
			ctx,
			int32(1),
		).Return(&expectedWp, nil)

		// check users
		userRepo.EXPECT().GetByIDs(
			ctx,
			[]string{testUUIDString},
		).Return([]*entities.User{testUserTbz}, nil)

		// check treecluster
		clusterRepo.EXPECT().GetByIDs(
			ctx,
			[]int32{1, 2},
		).Return(allTestClusters[0:2], nil)

		// check transporter
		vehicleRepo.EXPECT().GetByID(
			ctx,
			int32(2),
		).Return(allTestVehicles[1], nil)

		wateringPlanRepo.EXPECT().Update(
			ctx,
			expectedWp.ID,
			mock.Anything,
		).Return(nil)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, eventManager, routingRepo, s3Repo)

		// when
		subID, ch, err := eventManager.Subscribe(entities.EventTypeUpdateWateringPlan)
		if err != nil {
			t.Fatal("failed to subscribe to event manager")
		}
		_, err = svc.Update(ctx, expectedWp.ID, updatedWateringPlan)

		// then
		assert.NoError(t, err)
		select {
		case recievedEvent := <-ch:
			assert.Equal(t, recievedEvent, expectedEvent)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}

		_ = eventManager.Unsubscribe(entities.EventTypeUpdateWateringPlan, subID)
	})
}

func TestWateringPlanService_Delete(t *testing.T) {
	ctx := context.Background()

	wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
	clusterRepo := storageMock.NewMockTreeClusterRepository(t)
	vehicleRepo := storageMock.NewMockVehicleRepository(t)
	userRepo := storageMock.NewMockUserRepository(t)
	routingRepo := storageMock.NewMockRoutingRepository(t)
	s3Repo := storageMock.NewMockS3Repository(t)

	svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

	t.Run("should successfully delete a watering plan", func(t *testing.T) {
		id := int32(1)

		wateringPlanRepo.EXPECT().GetByID(ctx, id).Return(allTestWateringPlans[1], nil)
		wateringPlanRepo.EXPECT().Delete(ctx, id).Return(nil)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.NoError(t, err)
	})

	t.Run("should return error if watering plan not found", func(t *testing.T) {
		id := int32(2)

		wateringPlanRepo.EXPECT().GetByID(ctx, id).Return(nil, storage.ErrEntityNotFound("not found"))

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: watering plan not found")
	})

	t.Run("should return error if deleting watering plan fails", func(t *testing.T) {
		id := int32(4)

		wateringPlanRepo.EXPECT().GetByID(ctx, id).Return(allTestWateringPlans[1], nil)
		expectedErr := errors.New("failed to delete")
		wateringPlanRepo.EXPECT().Delete(ctx, id).Return(expectedErr)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to delete")
	})
}

func TestWateringPlanService_UpdateStatuses(t *testing.T) {
	t.Run("should update not competed watering plans successfully", func(t *testing.T) {
		// given
		ctx := context.Background()
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		// should be updated
		stalePlanActive := &entities.WateringPlan{
			ID:     1,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: entities.WateringPlanStatusActive,
		}
		stalePlanPlanned := &entities.WateringPlan{
			ID:     2,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: entities.WateringPlanStatusPlanned,
		}
		stalePlanUnknown := &entities.WateringPlan{
			ID:     3,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: entities.WateringPlanStatusUnknown,
		}

		// should not be updated
		stalePlanNotCompeted := &entities.WateringPlan{
			ID:     4,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: entities.WateringPlanStatusNotCompeted,
		}
		stalePlanFinished := &entities.WateringPlan{
			ID:     5,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: entities.WateringPlanStatusFinished,
		}
		recentPlanActive := &entities.WateringPlan{
			ID:     6,
			Date:   time.Now(),
			Status: entities.WateringPlanStatusActive,
		}

		expectList := []*entities.WateringPlan{
			stalePlanActive,
			stalePlanPlanned,
			stalePlanUnknown,
			stalePlanNotCompeted,
			stalePlanFinished,
			recentPlanActive,
		}

		// when
		wateringPlanRepo.EXPECT().GetAll(mock.Anything, entities.Query{}).Return(expectList, int64(len(expectList)), nil)
		wateringPlanRepo.EXPECT().Update(mock.Anything, stalePlanActive.ID, mock.Anything).Return(nil)
		wateringPlanRepo.EXPECT().Update(mock.Anything, stalePlanPlanned.ID, mock.Anything).Return(nil)
		wateringPlanRepo.EXPECT().Update(mock.Anything, stalePlanUnknown.ID, mock.Anything).Return(nil)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.NoError(t, err)
		wateringPlanRepo.AssertCalled(t, "GetAll", mock.Anything, entities.Query{})
		wateringPlanRepo.AssertCalled(t, "Update", mock.Anything, stalePlanActive.ID, mock.Anything)
		wateringPlanRepo.AssertCalled(t, "Update", mock.Anything, stalePlanPlanned.ID, mock.Anything)
		wateringPlanRepo.AssertCalled(t, "Update", mock.Anything, stalePlanUnknown.ID, mock.Anything)
		wateringPlanRepo.AssertNotCalled(t, "Update", mock.Anything, stalePlanNotCompeted.ID, mock.Anything)
		wateringPlanRepo.AssertNotCalled(t, "Update", mock.Anything, stalePlanFinished.ID, mock.Anything)
		wateringPlanRepo.AssertNotCalled(t, "Update", mock.Anything, recentPlanActive.ID, mock.Anything)
		wateringPlanRepo.AssertExpectations(t)
	})

	t.Run("should do nothing when there are no stale watering plans", func(t *testing.T) {
		// given
		ctx := context.Background()
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		recentPlanActive := &entities.WateringPlan{
			ID:     6,
			Date:   time.Now(),
			Status: entities.WateringPlanStatusActive,
		}

		expectList := []*entities.WateringPlan{recentPlanActive}

		// when
		wateringPlanRepo.EXPECT().GetAll(mock.Anything, entities.Query{}).Return(expectList, int64(len(expectList)), nil)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.NoError(t, err)
		wateringPlanRepo.AssertCalled(t, "GetAll", mock.Anything, entities.Query{})
		wateringPlanRepo.AssertNotCalled(t, "Update")
		wateringPlanRepo.AssertExpectations(t)
	})

	t.Run("should return an error when fetching watering plans fails", func(t *testing.T) {
		// given
		ctx := context.Background()
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		// when
		expectedErr := errors.New("database error")
		wateringPlanRepo.EXPECT().GetAll(mock.Anything, entities.Query{}).Return(nil, int64(0), expectedErr)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.Error(t, err)
		assert.Equal(t, expectedErr, err)
		wateringPlanRepo.AssertCalled(t, "GetAll", mock.Anything, entities.Query{})
		wateringPlanRepo.AssertNotCalled(t, "Update")
		wateringPlanRepo.AssertExpectations(t)
	})

	t.Run("should log an error when updating a watering plan fails", func(t *testing.T) {
		// given
		ctx := context.Background()
		wateringPlanRepo := storageMock.NewMockWateringPlanRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		routingRepo := storageMock.NewMockRoutingRepository(t)
		s3Repo := storageMock.NewMockS3Repository(t)

		svc := NewWateringPlanService(wateringPlanRepo, clusterRepo, vehicleRepo, userRepo, globalEventManager, routingRepo, s3Repo)

		stalePlanUnknown := &entities.WateringPlan{
			ID:     3,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: entities.WateringPlanStatusUnknown,
		}

		expectList := []*entities.WateringPlan{stalePlanUnknown}

		// when
		wateringPlanRepo.EXPECT().GetAll(mock.Anything, entities.Query{}).Return(expectList, int64(len(expectList)), nil)
		wateringPlanRepo.EXPECT().Update(mock.Anything, stalePlanUnknown.ID, mock.Anything).Return(errors.New("update failed"))

		err := svc.UpdateStatuses(ctx)

		// then
		wateringPlanRepo.AssertCalled(t, "GetAll", mock.Anything, entities.Query{})
		wateringPlanRepo.AssertCalled(t, "Update", mock.Anything, stalePlanUnknown.ID, mock.Anything)
		wateringPlanRepo.AssertExpectations(t)
		assert.NoError(t, err)
	})
}

var allTestWateringPlans = []*entities.WateringPlan{
	{
		ID:                 1,
		Date:               time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
		Description:        "New watering plan for the west side of the city",
		Status:             entities.WateringPlanStatusPlanned,
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(6000.0),
		Transporter:        allTestVehicles[1],
		Trailer:            allTestVehicles[0],
		TreeClusters:       allTestClusters[0:2],
		CancellationNote:   "",
	},
	{
		ID:                 2,
		Date:               time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
		Description:        "New watering plan for the east side of the city",
		Status:             entities.WateringPlanStatusActive,
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(6000.0),
		Transporter:        allTestVehicles[1],
		Trailer:            allTestVehicles[0],
		TreeClusters:       allTestClusters[2:3],
		CancellationNote:   "",
	},
	{
		ID:                 3,
		Date:               time.Date(2024, 6, 12, 0, 0, 0, 0, time.UTC),
		Description:        "Very important watering plan due to no rainfall",
		Status:             entities.WateringPlanStatusFinished,
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(6000.0),
		Transporter:        allTestVehicles[1],
		Trailer:            nil,
		TreeClusters:       allTestClusters[0:3],
		CancellationNote:   "",
		Evaluation: []*entities.EvaluationValue{
			{
				WateringPlanID: 3,
				TreeClusterID:  1,
				ConsumedWater:  utils.P(10.0),
			},
			{
				WateringPlanID: 3,
				TreeClusterID:  2,
				ConsumedWater:  utils.P(10.0),
			},
			{
				WateringPlanID: 3,
				TreeClusterID:  3,
				ConsumedWater:  utils.P(10.0),
			},
		},
	},
	{
		ID:                 4,
		Date:               time.Date(2024, 6, 10, 0, 0, 0, 0, time.UTC),
		Description:        "New watering plan for the south side of the city",
		Status:             entities.WateringPlanStatusNotCompeted,
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(6000.0),
		Transporter:        allTestVehicles[1],
		Trailer:            nil,
		TreeClusters:       allTestClusters[2:3],
		CancellationNote:   "",
	},
	{
		ID:                 5,
		Date:               time.Date(2024, 6, 4, 0, 0, 0, 0, time.UTC),
		Description:        "Canceled due to flood",
		Status:             entities.WateringPlanStatusCanceled,
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(6000.0),
		Transporter:        allTestVehicles[1],
		Trailer:            nil,
		TreeClusters:       allTestClusters[2:3],
		CancellationNote:   "The watering plan was cancelled due to various reasons.",
	},
}

var allTestVehicles = []*entities.Vehicle{
	{
		ID:             1,
		NumberPlate:    "B-1234",
		Description:    "Test vehicle 1",
		DrivingLicense: entities.DrivingLicenseBE,
		WaterCapacity:  100.0,
		Type:           entities.VehicleTypeTrailer,
		Status:         entities.VehicleStatusActive,
	},
	{
		ID:             2,
		NumberPlate:    "B-5678",
		Description:    "Test vehicle 2",
		DrivingLicense: entities.DrivingLicenseC,
		WaterCapacity:  150.0,
		Type:           entities.VehicleTypeTransporter,
		Status:         entities.VehicleStatusUnknown,
	},
}

var allTestClusters = []*entities.TreeCluster{
	{
		ID:             1,
		Name:           "Solitüde Strand",
		WateringStatus: entities.WateringStatusGood,
		MoistureLevel:  0.75,
		Region: &entities.Region{
			ID:   1,
			Name: "Mürwik",
		},
		Address:       "Solitüde Strand",
		Description:   "Alle Bäume am Strand",
		SoilCondition: entities.TreeSoilConditionSandig,
		Latitude:      utils.P(54.820940),
		Longitude:     utils.P(9.489022),
		Trees: []*entities.Tree{
			{ID: 1},
			{ID: 2},
			{ID: 3},
		},
	},
	{
		ID:             2,
		Name:           "Sankt-Jürgen-Platz",
		WateringStatus: entities.WateringStatusModerate,
		MoistureLevel:  0.5,
		Region: &entities.Region{
			ID:   1,
			Name: "Mürwik",
		},
		Address:       "Ulmenstraße",
		Description:   "Bäume beim Sankt-Jürgen-Platz",
		SoilCondition: entities.TreeSoilConditionSchluffig,
		Latitude:      utils.P(54.78805731048199),
		Longitude:     utils.P(9.44400186680097),
		Trees: []*entities.Tree{
			{ID: 4},
			{ID: 5},
			{ID: 6},
		},
	},
	{
		ID:             3,
		Name:           "Flensburger Stadion",
		WateringStatus: "unknown",
		MoistureLevel:  0.7,
		Region: &entities.Region{
			ID:   1,
			Name: "Mürwik",
		},
		Address:       "Flensburger Stadion",
		Description:   "Alle Bäume in der Gegend des Stadions in Mürwik",
		SoilCondition: "schluffig",
		Latitude:      utils.P(54.802163),
		Longitude:     utils.P(9.446398),
		Trees:         []*entities.Tree{},
	},
}

var testUserTbz = &entities.User{
	Roles: []entities.UserRole{entities.UserRoleTbz},
	DrivingLicenses: []entities.DrivingLicense{
		entities.DrivingLicenseB,
		entities.DrivingLicenseBE,
		entities.DrivingLicenseC,
		entities.DrivingLicenseCE,
	},
}

var testUserGreenEcolution = &entities.User{
	Roles: []entities.UserRole{entities.UserRoleGreenEcolution},
	DrivingLicenses: []entities.DrivingLicense{
		entities.DrivingLicenseB,
		entities.DrivingLicenseBE,
		entities.DrivingLicenseC,
		entities.DrivingLicenseCE,
	},
}

var testUserCar = &entities.User{
	Roles:           []entities.UserRole{entities.UserRoleTbz},
	DrivingLicenses: []entities.DrivingLicense{entities.DrivingLicenseB},
}
