package watering

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

var globalEventManager = worker.NewEventManager() //shared.EventTypeUpdateWateringPlan

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

		wateringPlanRepo.EXPECT().GetAll(ctx, shared.Query{}).Return(allTestWateringPlans, int64(len(allTestWateringPlans)), nil)

		// when
		wateringPlans, totalCount, err := svc.GetAll(ctx, shared.Query{})

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

		wateringPlanRepo.EXPECT().GetAll(ctx, shared.Query{Provider: "test-provider"}).Return(allTestWateringPlans, int64(len(allTestWateringPlans)), nil)

		// when
		wateringPlans, totalCount, err := svc.GetAll(ctx, shared.Query{Provider: "test-provider"})

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

		wateringPlanRepo.EXPECT().GetAll(ctx, shared.Query{}).Return([]*shared.WateringPlan{}, int64(0), nil)

		// when
		wateringPlans, totalCount, err := svc.GetAll(ctx, shared.Query{})

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
		wateringPlanRepo.EXPECT().GetAll(ctx, shared.Query{}).Return(nil, int64(0), expectedErr)

		// when
		wateringPlans, totalCount, err := svc.GetAll(ctx, shared.Query{})

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
		expectedErr := shared.ErrEntityNotFound("not found")
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

	futureDate := time.Now().Add(24 * time.Hour).Truncate(24 * time.Hour)

	newWateringPlan := &shared.WateringPlanCreate{
		Date:           futureDate,
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
		).Return([]*shared.User{testUserTbz}, nil)

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

		newWateringPlan := &shared.WateringPlanCreate{
			Date:           futureDate,
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
		).Return([]*shared.User{testUserTbz}, nil)

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
		).Return(nil, shared.ErrConnectionClosed)

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
		).Return([]*shared.TreeCluster{}, nil)

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
		).Return(nil, shared.ErrVehicleNotFound)

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
		).Return([]*shared.User{}, nil)

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
		).Return(nil, shared.ErrUserNotFound)

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
		).Return([]*shared.User{testUserGreenEcolution}, nil)

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
		).Return([]*shared.User{{Roles: []shared.UserRole{}}}, nil)

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
		).Return([]*shared.User{testUserCar}, nil)

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
		).Return([]*shared.User{testUserTbz}, nil)

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

}

func TestWateringPlanService_Update(t *testing.T) {
	ctx := context.Background()
	testUUIDString := "6a1078e8-80fd-458f-b74e-e388fe2dd6ab"
	testUUID, err := uuid.Parse(testUUIDString)
	if err != nil {
		t.Fatal(err)
	}

	futureDate := time.Now().Add(24 * time.Hour).Truncate(24 * time.Hour)

	updatedWateringPlan := &shared.WateringPlanUpdate{
		Date:             futureDate,
		Description:      "New watering plan for the east side of the city",
		TransporterID:    utils.P(int32(2)),
		TrailerID:        utils.P(int32(1)),
		TreeClusterIDs:   []*int32{utils.P(int32(1)), utils.P(int32(2))},
		UserIDs:          []*uuid.UUID{&testUUID},
		Status:           shared.WateringPlanStatusActive,
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
		).Return([]*shared.User{testUserTbz}, nil)

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

		updatedWateringPlan := &shared.WateringPlanUpdate{
			Date:             futureDate,
			Status:           shared.WateringPlanStatusFinished,
			CancellationNote: "",
			Description:      "New watering plan for the east side of the city",
			TransporterID:    utils.P(int32(2)),
			TreeClusterIDs:   []*int32{utils.P(int32(1)), utils.P(int32(2))},
			UserIDs:          []*uuid.UUID{&testUUID},
			Evaluation: []*shared.EvaluationValue{
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
		).Return([]*shared.User{testUserTbz}, nil)

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

		updatedWateringPlan := &shared.WateringPlanUpdate{
			Date:             futureDate,
			Status:           shared.WateringPlanStatusActive,
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
		).Return([]*shared.User{testUserTbz}, nil)

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
		).Return(nil, shared.ErrConnectionClosed)

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
		).Return([]*shared.TreeCluster{}, nil)

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
		).Return(nil, shared.ErrVehicleNotFound)

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
		).Return([]*shared.User{}, nil)

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
		).Return([]*shared.User{testUserGreenEcolution}, nil)

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
		).Return([]*shared.User{{Roles: []shared.UserRole{}}}, nil)

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
		).Return(nil, shared.ErrUserNotFound)

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
		).Return([]*shared.User{testUserCar}, nil)

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
		).Return([]*shared.User{testUserTbz}, nil)

		wateringPlanRepo.EXPECT().Update(
			ctx,
			int32(1),
			mock.Anything,
		).Return(shared.ErrEntityNotFound("not found"))

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
		).Return([]*shared.User{testUserTbz}, nil)

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

		futureDate := time.Now().Add(24 * time.Hour).Truncate(24 * time.Hour)

		prevWp := shared.WateringPlan{
			ID:           1,
			Date:         futureDate,
			TreeClusters: []*shared.TreeCluster{{ID: 1}},
			Status:       shared.WateringPlanStatusActive,
			UserIDs:      []*uuid.UUID{&testUUID},
		}

		updatedWateringPlan := &shared.WateringPlanUpdate{
			Date:           futureDate,
			TransporterID:  utils.P(int32(2)),
			TreeClusterIDs: []*int32{utils.P(int32(1)), utils.P(int32(2))},
			UserIDs:        []*uuid.UUID{&testUUID},
			Status:         shared.WateringPlanStatusActive,
		}

		expectedWp := shared.WateringPlan{
			ID:           1,
			Date:         futureDate,
			TreeClusters: []*shared.TreeCluster{{ID: 1}},
			Status:       shared.WateringPlanStatusActive,
			UserIDs:      []*uuid.UUID{&testUUID},
		}

		// Event
		eventManager := worker.NewEventManager(shared.EventTypeUpdateWateringPlan)
		expectedEvent := shared.NewEventUpdateWateringPlan(&prevWp, &expectedWp)
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
		).Return([]*shared.User{testUserTbz}, nil)

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
		subID, ch, err := eventManager.Subscribe(shared.EventTypeUpdateWateringPlan)
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

		_ = eventManager.Unsubscribe(shared.EventTypeUpdateWateringPlan, subID)
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

		wateringPlanRepo.EXPECT().GetByID(ctx, id).Return(nil, shared.ErrEntityNotFound("not found"))

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
		stalePlanActive := &shared.WateringPlan{
			ID:     1,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: shared.WateringPlanStatusActive,
		}
		stalePlanPlanned := &shared.WateringPlan{
			ID:     2,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: shared.WateringPlanStatusPlanned,
		}
		stalePlanUnknown := &shared.WateringPlan{
			ID:     3,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: shared.WateringPlanStatusUnknown,
		}

		// should not be updated
		stalePlanNotCompeted := &shared.WateringPlan{
			ID:     4,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: shared.WateringPlanStatusNotCompeted,
		}
		stalePlanFinished := &shared.WateringPlan{
			ID:     5,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: shared.WateringPlanStatusFinished,
		}
		recentPlanActive := &shared.WateringPlan{
			ID:     6,
			Date:   time.Now(),
			Status: shared.WateringPlanStatusActive,
		}

		expectList := []*shared.WateringPlan{
			stalePlanActive,
			stalePlanPlanned,
			stalePlanUnknown,
			stalePlanNotCompeted,
			stalePlanFinished,
			recentPlanActive,
		}

		// when
		wateringPlanRepo.EXPECT().GetAll(mock.Anything, shared.Query{}).Return(expectList, int64(len(expectList)), nil)
		wateringPlanRepo.EXPECT().Update(mock.Anything, stalePlanActive.ID, mock.Anything).Return(nil)
		wateringPlanRepo.EXPECT().Update(mock.Anything, stalePlanPlanned.ID, mock.Anything).Return(nil)
		wateringPlanRepo.EXPECT().Update(mock.Anything, stalePlanUnknown.ID, mock.Anything).Return(nil)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.NoError(t, err)
		wateringPlanRepo.AssertCalled(t, "GetAll", mock.Anything, shared.Query{})
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

		recentPlanActive := &shared.WateringPlan{
			ID:     6,
			Date:   time.Now(),
			Status: shared.WateringPlanStatusActive,
		}

		expectList := []*shared.WateringPlan{recentPlanActive}

		// when
		wateringPlanRepo.EXPECT().GetAll(mock.Anything, shared.Query{}).Return(expectList, int64(len(expectList)), nil)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.NoError(t, err)
		wateringPlanRepo.AssertCalled(t, "GetAll", mock.Anything, shared.Query{})
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
		wateringPlanRepo.EXPECT().GetAll(mock.Anything, shared.Query{}).Return(nil, int64(0), expectedErr)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.Error(t, err)
		assert.Equal(t, expectedErr, err)
		wateringPlanRepo.AssertCalled(t, "GetAll", mock.Anything, shared.Query{})
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

		stalePlanUnknown := &shared.WateringPlan{
			ID:     3,
			Date:   time.Now().Add(-73 * time.Hour),
			Status: shared.WateringPlanStatusUnknown,
		}

		expectList := []*shared.WateringPlan{stalePlanUnknown}

		// when
		wateringPlanRepo.EXPECT().GetAll(mock.Anything, shared.Query{}).Return(expectList, int64(len(expectList)), nil)
		wateringPlanRepo.EXPECT().Update(mock.Anything, stalePlanUnknown.ID, mock.Anything).Return(errors.New("update failed"))

		err := svc.UpdateStatuses(ctx)

		// then
		wateringPlanRepo.AssertCalled(t, "GetAll", mock.Anything, shared.Query{})
		wateringPlanRepo.AssertCalled(t, "Update", mock.Anything, stalePlanUnknown.ID, mock.Anything)
		wateringPlanRepo.AssertExpectations(t)
		assert.NoError(t, err)
	})
}

var allTestWateringPlans = []*shared.WateringPlan{
	{
		ID:                 1,
		Date:               time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
		Description:        "New watering plan for the west side of the city",
		Status:             shared.WateringPlanStatusPlanned,
		Distance:           utils.P(shared.MustNewDistance(63.0)),
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
		Status:             shared.WateringPlanStatusActive,
		Distance:           utils.P(shared.MustNewDistance(63.0)),
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
		Status:             shared.WateringPlanStatusFinished,
		Distance:           utils.P(shared.MustNewDistance(63.0)),
		TotalWaterRequired: utils.P(6000.0),
		Transporter:        allTestVehicles[1],
		Trailer:            nil,
		TreeClusters:       allTestClusters[0:3],
		CancellationNote:   "",
		Evaluation: []*shared.EvaluationValue{
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
		Status:             shared.WateringPlanStatusNotCompeted,
		Distance:           utils.P(shared.MustNewDistance(63.0)),
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
		Status:             shared.WateringPlanStatusCanceled,
		Distance:           utils.P(shared.MustNewDistance(63.0)),
		TotalWaterRequired: utils.P(6000.0),
		Transporter:        allTestVehicles[1],
		Trailer:            nil,
		TreeClusters:       allTestClusters[2:3],
		CancellationNote:   "The watering plan was cancelled due to various reasons.",
	},
}

var allTestVehicles = []*shared.Vehicle{
	{
		ID:             1,
		NumberPlate:    "B-1234",
		Description:    "Test vehicle 1",
		DrivingLicense: shared.DrivingLicenseBE,
		WaterCapacity:  shared.MustNewWaterCapacity(100.0),
		Type:           shared.VehicleTypeTrailer,
		Status:         shared.VehicleStatusActive,
	},
	{
		ID:             2,
		NumberPlate:    "B-5678",
		Description:    "Test vehicle 2",
		DrivingLicense: shared.DrivingLicenseC,
		WaterCapacity:  shared.MustNewWaterCapacity(150.0),
		Type:           shared.VehicleTypeTransporter,
		Status:         shared.VehicleStatusUnknown,
	},
}

var (
	allTestCluster1Coord = shared.MustNewCoordinate(54.820940, 9.489022)
	allTestCluster2Coord = shared.MustNewCoordinate(54.78805731048199, 9.44400186680097)
	allTestCluster3Coord = shared.MustNewCoordinate(54.802163, 9.446398)
)

var allTestClusters = []*shared.TreeCluster{
	{
		ID:             1,
		Name:           "Solitüde Strand",
		WateringStatus: shared.WateringStatusGood,
		MoistureLevel:  0.75,
		Region: &shared.Region{
			ID:   1,
			Name: "Mürwik",
		},
		Address:       "Solitüde Strand",
		Description:   "Alle Bäume am Strand",
		SoilCondition: shared.TreeSoilConditionSandig,
		Coordinate:    &allTestCluster1Coord,
		Trees: []*shared.Tree{
			{ID: 1},
			{ID: 2},
			{ID: 3},
		},
	},
	{
		ID:             2,
		Name:           "Sankt-Jürgen-Platz",
		WateringStatus: shared.WateringStatusModerate,
		MoistureLevel:  0.5,
		Region: &shared.Region{
			ID:   1,
			Name: "Mürwik",
		},
		Address:       "Ulmenstraße",
		Description:   "Bäume beim Sankt-Jürgen-Platz",
		SoilCondition: shared.TreeSoilConditionSchluffig,
		Coordinate:    &allTestCluster2Coord,
		Trees: []*shared.Tree{
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
		Region: &shared.Region{
			ID:   1,
			Name: "Mürwik",
		},
		Address:       "Flensburger Stadion",
		Description:   "Alle Bäume in der Gegend des Stadions in Mürwik",
		SoilCondition: "schluffig",
		Coordinate:    &allTestCluster3Coord,
		Trees:         []*shared.Tree{},
	},
}

var testUserTbz = &shared.User{
	Roles: []shared.UserRole{shared.UserRoleTbz},
	DrivingLicenses: []shared.DrivingLicense{
		shared.DrivingLicenseB,
		shared.DrivingLicenseBE,
		shared.DrivingLicenseC,
		shared.DrivingLicenseCE,
	},
}

var testUserGreenEcolution = &shared.User{
	Roles: []shared.UserRole{shared.UserRoleGreenEcolution},
	DrivingLicenses: []shared.DrivingLicense{
		shared.DrivingLicenseB,
		shared.DrivingLicenseBE,
		shared.DrivingLicenseC,
		shared.DrivingLicenseCE,
	},
}

var testUserCar = &shared.User{
	Roles:           []shared.UserRole{shared.UserRoleTbz},
	DrivingLicenses: []shared.DrivingLicense{shared.DrivingLicenseB},
}
