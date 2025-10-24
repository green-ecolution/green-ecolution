package treecluster

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/storage"
	storageMock "github.com/green-ecolution/backend/internal/storage/_mock"
	"github.com/green-ecolution/backend/internal/utils"
	"github.com/green-ecolution/backend/internal/worker"
	"github.com/stretchr/testify/assert"
	mock "github.com/stretchr/testify/mock"
)

var globalEventManager = worker.NewEventManager() //entities.EventTypeUpdateTree, entities.EventTypeUpdateTreeCluster

func TestTreeClusterService_GetAll(t *testing.T) {
	ctx := context.Background()

	t.Run("should return all tree clusters when successful", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		expectedClusters := testClusters
		clusterRepo.EXPECT().GetAll(ctx, entities.TreeClusterQuery{}).Return(expectedClusters, int64(len(expectedClusters)), nil)

		// when
		clusters, totalCount, err := svc.GetAll(ctx, entities.TreeClusterQuery{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedClusters, clusters)
		assert.Equal(t, totalCount, int64(len(expectedClusters)))
	})

	t.Run("should return all tree clusters when successful with provider", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		expectedClusters := testClusters
		clusterRepo.EXPECT().GetAll(ctx, entities.TreeClusterQuery{Query: entities.Query{Provider: "test-provider"}}).Return(expectedClusters, int64(len(expectedClusters)), nil)

		// when
		clusters, totalCount, err := svc.GetAll(ctx, entities.TreeClusterQuery{Query: entities.Query{Provider: "test-provider"}})

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedClusters, clusters)
		assert.Equal(t, totalCount, int64(len(expectedClusters)))
	})

	t.Run("should return empty slice when no clusters are found", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		clusterRepo.EXPECT().GetAll(ctx, entities.TreeClusterQuery{}).Return([]*entities.TreeCluster{}, int64(0), nil)

		// when
		clusters, totalCount, err := svc.GetAll(ctx, entities.TreeClusterQuery{})

		// then
		assert.NoError(t, err)
		assert.Empty(t, clusters)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error when GetAll fails", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		expectedErr := errors.New("GetAll failed")

		clusterRepo.EXPECT().GetAll(ctx, entities.TreeClusterQuery{}).Return(nil, int64(0), expectedErr)

		// when
		clusters, totalCount, err := svc.GetAll(ctx, entities.TreeClusterQuery{})

		// then
		assert.Error(t, err)
		assert.Nil(t, clusters)
		assert.Equal(t, totalCount, int64(0))
		// assert.EqualError(t, err, "500: GetAll failed")
	})
}

func TestTreeClusterService_GetByID(t *testing.T) {
	ctx := context.Background()

	clusterRepo := storageMock.NewMockTreeClusterRepository(t)
	treeRepo := storageMock.NewMockTreeRepository(t)
	regionRepo := storageMock.NewMockRegionRepository(t)
	svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

	t.Run("should return tree cluster when found", func(t *testing.T) {
		id := int32(1)
		expectedCluster := testClusters[0]
		clusterRepo.EXPECT().GetByID(ctx, id).Return(expectedCluster, nil)

		// when
		cluster, err := svc.GetByID(ctx, id)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedCluster, cluster)
	})

	t.Run("should return error if tree cluster not found", func(t *testing.T) {
		id := int32(2)
		expectedErr := storage.ErrEntityNotFound("not found")
		clusterRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedErr)

		// when
		cluster, err := svc.GetByID(ctx, id)

		// then
		assert.Error(t, err)
		assert.Nil(t, cluster)
		// assert.EqualError(t, err, "404: treecluster not found")
	})
}

func TestTreeClusterService_Create(t *testing.T) {
	ctx := context.Background()
	newCluster := &entities.TreeClusterCreate{
		Name:          "Cluster 1",
		Address:       "123 Main St",
		Description:   "Test description",
		SoilCondition: entities.TreeSoilConditionLehmig,
		TreeIDs:       []*int32{utils.P(int32(1)), utils.P(int32(2))},
	}

	t.Run("should successfully create a new tree cluster", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		expectedCluster := testClusters[0]

		treeRepo.EXPECT().GetTreesByIDs(
			ctx,
			[]int32{1, 2},
		).Return(testTrees, nil)

		clusterRepo.EXPECT().Create(
			ctx,
			mock.Anything,
		).Return(expectedCluster, nil)

		// UpdateWateringStatuses
		clusterRepo.EXPECT().GetAll(mock.Anything, entities.TreeClusterQuery{}).Return(testClusters, int64(len(testClusters)), nil)

		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(
			ctx,
			int32(1),
		).Return(allLatestSensorData, nil)

		treeRepo.EXPECT().GetBySensorIDs(
			ctx,
			"sensor-1",
		).Return(testTrees, nil)

		clusterRepo.EXPECT().Update(
			ctx,
			expectedCluster.ID,
			mock.Anything,
		).Return(nil)

		// when
		result, err := svc.Create(ctx, newCluster)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedCluster, result)
	})

	t.Run("should successfully create a new tree cluster with empty trees", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		newCluster := &entities.TreeClusterCreate{
			Name:          "Cluster 1",
			Address:       "123 Main St",
			Description:   "Test description",
			SoilCondition: entities.TreeSoilConditionLehmig,
			TreeIDs:       []*int32{},
		}

		expectedCluster := testClusters[1]

		treeRepo.EXPECT().GetTreesByIDs(
			ctx,
			[]int32{},
		).Return(nil, nil)

		clusterRepo.EXPECT().Create(
			ctx,
			mock.Anything,
		).Return(expectedCluster, nil)

		// UpdateWateringStatuses
		clusterRepo.EXPECT().GetAll(mock.Anything, entities.TreeClusterQuery{}).Return(testClusters, int64(len(testClusters)), nil)

		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(
			ctx,
			int32(2),
		).Return(nil, storage.ErrSensorNotFound)

		clusterRepo.EXPECT().Update(
			ctx,
			expectedCluster.ID,
			mock.Anything,
		).Return(nil)

		// when
		result, err := svc.Create(ctx, newCluster)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedCluster, result)
	})

	t.Run("should return an error when getting trees fails", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		expectedErr := storage.ErrTreeNotFound

		treeRepo.EXPECT().GetTreesByIDs(
			ctx,
			[]int32{1, 2},
		).Return(nil, expectedErr)

		// when
		result, err := svc.Create(ctx, newCluster)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: tree not found")
	})

	t.Run("should return an error when creating cluster fails", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		expectedErr := errors.New("Failed to create cluster")
		expectedTrees := testTrees

		treeRepo.EXPECT().GetTreesByIDs(
			ctx,
			[]int32{1, 2},
		).Return(expectedTrees, nil)

		clusterRepo.EXPECT().Create(
			ctx,
			mock.Anything,
		).Return(nil, expectedErr)

		// when
		result, err := svc.Create(ctx, newCluster)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: Failed to create cluster")
	})

	t.Run("should return an error when creating cluster fails due to error in position update", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		expectedCluster := testClusters[0]
		expectedErr := errors.New("Failed to create cluster")
		expectedTrees := testTrees

		treeRepo.EXPECT().GetTreesByIDs(
			ctx,
			[]int32{1, 2},
		).Return(expectedTrees, nil)

		clusterRepo.EXPECT().Create(
			ctx,
			mock.Anything,
		).Return(expectedCluster, nil)

		// UpdateWateringStatuses
		clusterRepo.EXPECT().GetAll(mock.Anything, entities.TreeClusterQuery{}).Return(testClusters, int64(len(testClusters)), nil)

		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(
			ctx,
			int32(1),
		).Return(allLatestSensorData, nil)

		treeRepo.EXPECT().GetBySensorIDs(
			ctx,
			"sensor-1",
		).Return(testTrees, nil)

		clusterRepo.EXPECT().Update(
			ctx,
			expectedCluster.ID,
			mock.Anything,
		).Return(expectedErr)

		// when
		result, err := svc.Create(ctx, newCluster)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: Failed to create cluster")
	})

	t.Run("should return validation error on empty name", func(t *testing.T) {
		// given
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		newCluster := &entities.TreeClusterCreate{
			Name:          "",
			Address:       "123 Main St",
			Description:   "Test description",
			SoilCondition: entities.TreeSoilConditionLehmig,
			TreeIDs:       []*int32{},
		}

		// when
		result, err := svc.Create(ctx, newCluster)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		//assert.EqualError(t, err, "400: validation error: Key: 'TreeClusterCreate.Name' Error:Field validation for 'Name' failed on the 'required' tag")
	})
}

func TestTreeClusterService_Update(t *testing.T) {
	ctx := context.Background()
	clusterID := int32(1)
	updatedCluster := &entities.TreeClusterUpdate{
		Name:          "Cluster 1",
		Address:       "123 Main St",
		Description:   "Test description",
		SoilCondition: entities.TreeSoilConditionLehmig,
		TreeIDs:       []*int32{utils.P(int32(1)), utils.P(int32(2))},
	}

	t.Run("should successfully update a tree cluster", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		expectedCluster := testClusters[0]
		expectedTrees := testTrees

		treeRepo.EXPECT().GetTreesByIDs(
			ctx,
			[]int32{1, 2},
		).Return(expectedTrees, nil)

		clusterRepo.EXPECT().GetByID(ctx, clusterID).Return(expectedCluster, nil)

		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(
			ctx,
			int32(1),
		).Return(allLatestSensorData, nil)

		treeRepo.EXPECT().GetBySensorIDs(
			ctx,
			"sensor-1",
		).Return(testTrees, nil)

		clusterRepo.EXPECT().Update(
			ctx,
			clusterID,
			mock.Anything,
		).Return(nil)

		// UpdateWateringStatuses
		clusterRepo.EXPECT().GetAll(mock.Anything, entities.TreeClusterQuery{}).Return(testClusters, int64(len(testClusters)), nil)

		// when
		result, err := svc.Update(ctx, clusterID, updatedCluster)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedCluster, result)
	})

	t.Run("should successfully update a tree cluster with empty tree IDs", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		updatedClusterEmptyTrees := &entities.TreeClusterUpdate{
			Name:          "Cluster 1",
			Address:       "123 Main St",
			Description:   "Test description",
			SoilCondition: entities.TreeSoilConditionLehmig,
			TreeIDs:       []*int32{},
		}

		expectedCluster := testClusters[1]

		treeRepo.EXPECT().GetTreesByIDs(
			ctx,
			[]int32{},
		).Return(nil, nil)

		clusterRepo.EXPECT().GetByID(ctx, expectedCluster.ID).Return(expectedCluster, nil)

		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(
			ctx,
			int32(2),
		).Return(nil, storage.ErrSensorNotFound)

		clusterRepo.EXPECT().Update(
			ctx,
			expectedCluster.ID,
			mock.Anything,
		).Return(nil)

		// UpdateWateringStatuses
		clusterRepo.EXPECT().GetAll(mock.Anything, entities.TreeClusterQuery{}).Return(testClusters, int64(len(testClusters)), nil)

		// when
		result, err := svc.Update(ctx, expectedCluster.ID, updatedClusterEmptyTrees)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedCluster, result)
	})

	t.Run("should return an error when no trees are found", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		treeRepo.EXPECT().GetTreesByIDs(
			ctx,
			[]int32{1, 2},
		).Return(nil, storage.ErrTreeNotFound)

		// when
		result, err := svc.Update(context.Background(), clusterID, updatedCluster)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: tree not found")
	})

	t.Run("should return an error when the update fails", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		expectedErr := errors.New("failed to update cluster")
		expectedTrees := testTrees

		treeRepo.EXPECT().GetTreesByIDs(
			ctx,
			[]int32{1, 2},
		).Return(expectedTrees, nil)

		clusterRepo.EXPECT().Update(
			ctx,
			clusterID,
			mock.Anything,
		).Return(expectedErr)
		clusterRepo.EXPECT().GetByID(ctx, clusterID).Return(nil, nil)

		// when
		result, err := svc.Update(context.Background(), clusterID, updatedCluster)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to update cluster")
	})

	t.Run("should return an error when cluster ID does not exist", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		treeRepo.EXPECT().GetTreesByIDs(
			ctx,
			[]int32{1, 2},
		).Return(testTrees, nil)
		clusterRepo.EXPECT().GetByID(ctx, clusterID).Return(nil, nil)

		clusterRepo.EXPECT().Update(
			ctx,
			clusterID,
			mock.Anything,
		).Return(storage.ErrEntityNotFound("not found"))

		// when
		result, err := svc.Update(ctx, clusterID, updatedCluster)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: treecluster not found")
	})

	t.Run("should return validation error on empty name", func(t *testing.T) {
		// given
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		updateCluster := &entities.TreeClusterUpdate{
			Name:          "",
			Address:       "123 Main St",
			Description:   "Test description",
			SoilCondition: entities.TreeSoilConditionLehmig,
			TreeIDs:       []*int32{},
		}

		// when
		result, err := svc.Update(ctx, int32(1), updateCluster)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.Contains(t, err.Error(), "400: validation error")
	})
}

func TestTreeClusterService_EventSystem(t *testing.T) {
	t.Run("should send update treecluster event on update tree cluster", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)

		clusters := testClusters
		prevCluster := *clusters[1]
		updatedClusterEmptyTrees := &entities.TreeClusterUpdate{
			Name:          "Cluster 1",
			Address:       "123 Main St",
			Description:   "Test description",
			SoilCondition: entities.TreeSoilConditionLehmig,
			TreeIDs:       []*int32{},
		}

		expectedCluster := *clusters[1]

		// Event
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTreeCluster)
		expectedEvent := entities.NewEventUpdateTreeCluster(&prevCluster, &expectedCluster)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		treeRepo.EXPECT().GetTreesByIDs(
			ctx,
			[]int32{},
		).Return(nil, nil)

		clusterRepo.EXPECT().GetByID(ctx, expectedCluster.ID).Return(&expectedCluster, nil)

		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(
			ctx,
			int32(2),
		).Return(allLatestSensorData, nil)

		treeRepo.EXPECT().GetBySensorIDs(
			ctx,
			"sensor-1",
		).Return(testTrees, nil)

		clusterRepo.EXPECT().Update(
			ctx,
			expectedCluster.ID,
			mock.Anything,
		).Return(nil)

		// UpdateWateringStatuses
		clusterRepo.EXPECT().GetAll(mock.Anything, entities.TreeClusterQuery{}).Return(testClusters, int64(len(testClusters)), nil)

		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		// when
		subID, ch, err := eventManager.Subscribe(entities.EventTypeUpdateTreeCluster)
		if err != nil {
			t.Fatal("failed to subscribe to event manager")
		}
		_, err = svc.Update(ctx, expectedCluster.ID, updatedClusterEmptyTrees)

		// then
		assert.NoError(t, err)
		select {
		case recievedEvent := <-ch:
			assert.Equal(t, recievedEvent, expectedEvent)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}

		_ = eventManager.Unsubscribe(entities.EventTypeUpdateTreeCluster, subID)
	})
}

func TestTreeClusterService_Delete(t *testing.T) {
	ctx := context.Background()

	clusterRepo := storageMock.NewMockTreeClusterRepository(t)
	treeRepo := storageMock.NewMockTreeRepository(t)
	regionRepo := storageMock.NewMockRegionRepository(t)
	svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

	t.Run("should successfully delete a tree cluster", func(t *testing.T) {
		id := int32(1)

		clusterRepo.EXPECT().GetByID(ctx, id).Return(testClusters[0], nil)
		treeRepo.EXPECT().UnlinkTreeClusterID(ctx, id).Return(nil)
		clusterRepo.EXPECT().Delete(ctx, id).Return(nil)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.NoError(t, err)
		assert.Equal(t, nil, err)
	})

	t.Run("should return error if tree cluster not found", func(t *testing.T) {
		id := int32(2)

		expectedErr := storage.ErrEntityNotFound("not found")
		clusterRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedErr)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: treecluster not found")
	})

	t.Run("should return error if unlinking tree cluster ID fails", func(t *testing.T) {
		id := int32(3)
		expectedErr := errors.New("failed to unlink treecluster ID")

		clusterRepo.EXPECT().GetByID(ctx, id).Return(testClusters[0], nil)
		treeRepo.EXPECT().UnlinkTreeClusterID(ctx, id).Return(expectedErr)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to unlink treecluster ID")
	})

	t.Run("should return error if deleting tree cluster fails", func(t *testing.T) {
		id := int32(4)
		expectedErr := errors.New("failed to delete")

		clusterRepo.EXPECT().GetByID(ctx, id).Return(testClusters[0], nil)
		treeRepo.EXPECT().UnlinkTreeClusterID(ctx, id).Return(nil)
		clusterRepo.EXPECT().Delete(ctx, id).Return(expectedErr)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to delete")
	})
}

func TestTreeClusterService_UpdateWateringStatuses(t *testing.T) {
	t.Run("should update »just watered« watering status of tree cluster successfully", func(t *testing.T) {
		// given
		ctx := context.Background()
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		staleDate := time.Now().Add(-34 * time.Hour)
		recentDate := time.Now().Add(-2 * time.Hour)

		staleCluster := &entities.TreeCluster{
			ID:             1,
			LastWatered:    &staleDate, // Older than 24h
			Trees:          testTrees,
			WateringStatus: entities.WateringStatusJustWatered,
		}
		recentCluster := &entities.TreeCluster{
			ID:             2,
			LastWatered:    &recentDate,
			Trees:          testTrees,
			WateringStatus: entities.WateringStatusJustWatered,
		}
		expectList := []*entities.TreeCluster{staleCluster, recentCluster}

		// when
		clusterRepo.EXPECT().GetAll(mock.Anything, entities.TreeClusterQuery{}).Return(expectList, int64(len(expectList)), nil)
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, staleCluster.ID).Return(allLatestSensorData, nil)
		treeRepo.EXPECT().GetBySensorIDs(ctx, "sensor-1").Return(testTrees, nil)
		clusterRepo.EXPECT().Update(mock.Anything, staleCluster.ID, mock.Anything).Return(nil)

		err := svc.UpdateWateringStatuses(ctx)

		// then
		assert.NoError(t, err)
		clusterRepo.AssertCalled(t, "GetAll", mock.Anything, entities.TreeClusterQuery{})
		clusterRepo.AssertCalled(t, "GetAllLatestSensorDataByClusterID", mock.Anything, staleCluster.ID)
		treeRepo.AssertCalled(t, "GetBySensorIDs", mock.Anything, mock.Anything)
		clusterRepo.AssertCalled(t, "Update", mock.Anything, staleCluster.ID, mock.Anything)
		clusterRepo.AssertExpectations(t)
	})

	t.Run("should update watering status to unknown when tree cluster has no trees", func(t *testing.T) {
		// given
		ctx := context.Background()
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		staleDate := time.Now().Add(-34 * time.Hour)
		recentDate := time.Now().Add(-2 * time.Hour)

		staleCluster := &entities.TreeCluster{
			ID:             1,
			LastWatered:    &staleDate, // Older than 24h
			Trees:          nil,
			WateringStatus: entities.WateringStatusJustWatered,
		}
		recentCluster := &entities.TreeCluster{
			ID:             2,
			LastWatered:    &recentDate,
			Trees:          nil,
			WateringStatus: entities.WateringStatusJustWatered,
		}
		expectList := []*entities.TreeCluster{staleCluster, recentCluster}

		// when
		clusterRepo.EXPECT().GetAll(mock.Anything, entities.TreeClusterQuery{}).Return(expectList, int64(len(expectList)), nil)
		clusterRepo.EXPECT().Update(mock.Anything, staleCluster.ID, mock.Anything).Return(nil)
		clusterRepo.EXPECT().Update(mock.Anything, recentCluster.ID, mock.Anything).Return(nil)

		err := svc.UpdateWateringStatuses(ctx)

		// then
		assert.NoError(t, err)
		clusterRepo.AssertCalled(t, "GetAll", mock.Anything, entities.TreeClusterQuery{})
		clusterRepo.AssertCalled(t, "Update", mock.Anything, staleCluster.ID, mock.Anything)
		clusterRepo.AssertCalled(t, "Update", mock.Anything, recentCluster.ID, mock.Anything)
		clusterRepo.AssertExpectations(t)
	})

	t.Run("should return an error when fetching tree clusters fails", func(t *testing.T) {
		// given
		ctx := context.Background()
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		// when
		expectedErr := errors.New("database error")
		clusterRepo.EXPECT().GetAll(mock.Anything, entities.TreeClusterQuery{}).Return(nil, int64(0), expectedErr)

		err := svc.UpdateWateringStatuses(ctx)

		// then
		assert.Error(t, err)
		assert.Equal(t, expectedErr, err)
		clusterRepo.AssertCalled(t, "GetAll", mock.Anything, entities.TreeClusterQuery{})
		clusterRepo.AssertNotCalled(t, "GetAllLatestSensorDataByClusterID")
		clusterRepo.AssertNotCalled(t, "GetBySensorIDs")
		clusterRepo.AssertNotCalled(t, "Update")
		clusterRepo.AssertExpectations(t)
	})

	t.Run("should log an error when updating a treecluster fails", func(t *testing.T) {
		// given
		ctx := context.Background()
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		staleDate := time.Now().Add(-34 * time.Hour)
		staleCluster := &entities.TreeCluster{
			ID:             1,
			LastWatered:    &staleDate, // Older than 24h
			Trees:          testTrees,
			WateringStatus: entities.WateringStatusJustWatered,
		}
		expectList := []*entities.TreeCluster{staleCluster}

		// when
		clusterRepo.EXPECT().GetAll(mock.Anything, entities.TreeClusterQuery{}).Return(expectList, int64(len(expectList)), nil)
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, staleCluster.ID).Return(allLatestSensorData, nil)
		treeRepo.EXPECT().GetBySensorIDs(ctx, "sensor-1").Return(testTrees, nil)
		clusterRepo.EXPECT().Update(mock.Anything, staleCluster.ID, mock.Anything).Return(errors.New("update failed"))

		err := svc.UpdateWateringStatuses(ctx)

		// then
		clusterRepo.AssertCalled(t, "GetAll", mock.Anything, entities.TreeClusterQuery{})
		clusterRepo.AssertCalled(t, "GetAllLatestSensorDataByClusterID", mock.Anything, staleCluster.ID)
		treeRepo.AssertCalled(t, "GetBySensorIDs", mock.Anything, mock.Anything)
		clusterRepo.AssertCalled(t, "Update", mock.Anything, staleCluster.ID, mock.Anything)
		clusterRepo.AssertExpectations(t)
		assert.NoError(t, err)
	})
}

func TestReady(t *testing.T) {
	t.Run("should return true if the service is ready", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, globalEventManager)

		// when
		ready := svc.Ready()

		// then
		assert.True(t, ready)
	})

	t.Run("should return false if the service is not ready", func(t *testing.T) {
		svc := NewTreeClusterService(nil, nil, nil, nil)

		// when
		ready := svc.Ready()

		// then
		assert.False(t, ready)
	})
}

var testClusters = []*entities.TreeCluster{
	{
		ID:            1,
		CreatedAt:     time.Now(),
		UpdatedAt:     time.Now(),
		Name:          "Cluster 1",
		Address:       "123 Main St",
		Description:   "Test description",
		SoilCondition: entities.TreeSoilConditionLehmig,
		Archived:      false,
		Latitude:      utils.P(9.446741),
		Longitude:     utils.P(54.801539),
		Trees:         testTrees,
	},
	{
		ID:            2,
		CreatedAt:     time.Now(),
		UpdatedAt:     time.Now(),
		Name:          "Cluster 2",
		Address:       "456 Second St",
		Description:   "Test description",
		SoilCondition: entities.TreeSoilConditionSandig,
		Archived:      false,
		Latitude:      nil,
		Longitude:     nil,
		Trees:         testTrees,
	},
}

var testTrees = []*entities.Tree{
	{
		ID:           1,
		CreatedAt:    time.Now(),
		UpdatedAt:    time.Now(),
		Species:      "Oak",
		Number:       "T001",
		Latitude:     9.446741,
		Longitude:    54.801539,
		Description:  "A mature oak tree",
		PlantingYear: 2023,
		Sensor: &entities.Sensor{
			ID: "sensor-1",
		},
	},
	{
		ID:           2,
		CreatedAt:    time.Now(),
		UpdatedAt:    time.Now(),
		Species:      "Pine",
		Number:       "T002",
		Latitude:     9.446700,
		Longitude:    54.801510,
		Description:  "A young pine tree",
		PlantingYear: 2023,
		Sensor: &entities.Sensor{
			ID: "sensor-2",
		},
	},
}
