package tree_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/utils"

	"github.com/green-ecolution/green-ecolution/backend/internal/service/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
	"github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/storage/_mock"
	"github.com/stretchr/testify/assert"
)

var globalEventManager = worker.NewEventManager() //entities.EventTypeUpdateTree, entities.EventTypeUpdateTreeCluster

func TestTreeService_GetAll(t *testing.T) {
	ctx := context.Background()

	t.Run("should return all trees when successful", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedTrees := TestTreesList
		treeRepo.EXPECT().GetAll(ctx, entities.TreeQuery{}).Return(expectedTrees, int64(len(expectedTrees)), nil)

		// when
		trees, totalCount, err := svc.GetAll(ctx, entities.TreeQuery{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedTrees, trees)
		assert.Equal(t, totalCount, int64(len(expectedTrees)))
	})

	t.Run("should return all trees when successful with provider", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedTrees := TestTreesList
		treeRepo.EXPECT().GetAll(ctx, entities.TreeQuery{Query: entities.Query{Provider: "test-provider"}}).Return(expectedTrees, int64(len(expectedTrees)), nil)

		// when
		trees, totalCount, err := svc.GetAll(ctx, entities.TreeQuery{Query: entities.Query{Provider: "test-provider"}})

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedTrees, trees)
		assert.Equal(t, totalCount, int64(len(expectedTrees)))
	})

	t.Run("should return empty slice when no trees are found", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		treeRepo.EXPECT().GetAll(ctx, entities.TreeQuery{}).Return([]*entities.Tree{}, int64(0), nil)

		// when
		trees, totalCount, err := svc.GetAll(ctx, entities.TreeQuery{})

		// then
		assert.NoError(t, err)
		assert.Empty(t, trees)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error when GetAll fails", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedError := errors.New("GetAll failed")

		treeRepo.EXPECT().GetAll(ctx, entities.TreeQuery{}).Return(nil, int64(0), expectedError)

		// when
		trees, totalCount, err := svc.GetAll(ctx, entities.TreeQuery{})

		// then
		assert.Error(t, err)
		assert.Nil(t, trees)
		assert.Equal(t, totalCount, int64(0))
		// assert.EqualError(t, err, "500: GetAll failed")
	})

	t.Run("should return trees filtered by watering status and planting years", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedTrees := testFilterTrees
		treeRepo.EXPECT().GetAll(ctx, entities.TreeQuery{
			WateringStatuses: []entities.WateringStatus{entities.WateringStatusGood, entities.WateringStatusBad},
			PlantingYears:    []int32{2022, 2023},
			HasCluster:       utils.P(true),
		}).Return(expectedTrees, int64(len(expectedTrees)), nil)

		// when
		trees, totalCount, err := svc.GetAll(ctx, entities.TreeQuery{
			WateringStatuses: []entities.WateringStatus{entities.WateringStatusGood, entities.WateringStatusBad},
			PlantingYears:    []int32{2022, 2023},
			HasCluster:       utils.P(true),
		})

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedTrees, trees)
		assert.Equal(t, totalCount, int64(len(expectedTrees)))
	})
}

func TestTreeService_GetByID(t *testing.T) {
	ctx := context.Background()

	// Mocks
	treeRepo := storageMock.NewMockTreeRepository(t)
	sensorRepo := storageMock.NewMockSensorRepository(t)
	clusterRepo := storageMock.NewMockTreeClusterRepository(t)
	eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
	svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

	t.Run("should return tree when found", func(t *testing.T) {
		id := int32(1)
		expectedTree := TestTreesList[0]
		treeRepo.EXPECT().GetByID(ctx, id).Return(expectedTree, nil)

		// when
		tree, err := svc.GetByID(ctx, id)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedTree, tree)
	})

	t.Run("should return error if tree not found", func(t *testing.T) {
		id := int32(2)
		expectedError := storage.ErrEntityNotFound("not found")
		treeRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedError)

		// when
		tree, err := svc.GetByID(ctx, id)

		// then
		assert.Error(t, err)
		assert.Nil(t, tree)
		// assert.EqualError(t, err, "404: tree not found")
	})

	t.Run("should return error for unexpected repository error", func(t *testing.T) {
		id := int32(3)
		expectedError := errors.New("unexpected error")

		// Set expectation for GetByID
		treeRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedError)

		// when
		tree, err := svc.GetByID(ctx, id)

		// then
		assert.Error(t, err)
		assert.Nil(t, tree)
		// assert.EqualError(t, err, "500: unexpected error")
	})
}

func TestTreeService_GetBySensorID(t *testing.T) {
	ctx := context.Background()

	t.Run("should return tree when found", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		id := "sensor-1"
		expectedTree := TestTreesList[0]
		treeRepo.EXPECT().GetBySensorID(ctx, id).Return(expectedTree, nil)

		// when
		tree, err := svc.GetBySensorID(ctx, id)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedTree, tree)
	})

	t.Run("should return error if tree not found", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		id := "sensor-2"
		expectedError := storage.ErrEntityNotFound("not found")
		treeRepo.EXPECT().GetBySensorID(ctx, id).Return(nil, expectedError)

		// when
		tree, err := svc.GetBySensorID(ctx, id)

		// then
		assert.Error(t, err)
		assert.Nil(t, tree)
		// assert.EqualError(t, err, "404: tree not found")
	})

	t.Run("should return error if sensor not found", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		id := "sensor-2"
		expectedError := storage.ErrSensorNotFound
		treeRepo.EXPECT().GetBySensorID(ctx, id).Return(nil, expectedError)

		// when
		tree, err := svc.GetBySensorID(ctx, id)

		// then
		assert.Error(t, err)
		assert.Nil(t, tree)
		// assert.EqualError(t, err, "404: sensor not found")
	})

	t.Run("should return error for unexpected repository error", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		id := "sensor-3"
		expectedError := errors.New("unexpected error")

		treeRepo.EXPECT().GetBySensorID(ctx, id).Return(nil, expectedError)

		// when
		tree, err := svc.GetBySensorID(ctx, id)

		// then
		assert.Error(t, err)
		assert.Nil(t, tree)
		// assert.EqualError(t, err, "500: unexpected error")
	})
}

func TestTreeService_Create(t *testing.T) {
	ctx := context.Background()

	t.Run("should successfully create a new tree", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedTree := TestTreesList[0]
		expectedPrevSensorTree := TestTreesList[1]
		expectedCluster := TestTreeClusters[0]
		expectedSensor := TestSensors[0]

		if TestTreeCreate.TreeClusterID == nil {
			t.Fatal("TreeClusterID must not be nil for this test case")
		}
		if TestTreeCreate.SensorID == nil {
			t.Fatal("SensorID must not be nil for this test case")
		}

		treeRepo.EXPECT().Create(ctx, mock.Anything).RunAndReturn(
			func(ctx context.Context, fn func(*entities.Tree, storage.TreeRepository) (bool, error)) (*entities.Tree, error) {
				testTree := &entities.Tree{}

				clusterRepo.EXPECT().GetByID(ctx, *TestTreeCreate.TreeClusterID).Return(expectedCluster, nil)
				treeRepo.EXPECT().GetBySensorID(ctx, expectedSensor.ID).Return(expectedPrevSensorTree, nil)
				sensorRepo.EXPECT().GetByID(ctx, *TestTreeCreate.SensorID).Return(expectedSensor, nil)

				success, err := fn(testTree, treeRepo)
				if !success {
					return nil, err
				}

				return expectedTree, nil
			},
		)

		// when
		result, err := svc.Create(ctx, TestTreeCreate)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedTree, result)
	})

	t.Run("should return validation error", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)

		svc := tree.NewTreeService(treeRepo, sensorRepo, treeClusterRepo, globalEventManager)

		invalidTreeCreate := &entities.TreeCreate{
			Species:      "Oak",
			Latitude:     0,  // Invalid: must be between -90 and 90
			Longitude:    0,  // Invalid: must be between -180 and 180
			PlantingYear: 0,  // Invalid: PlantingYear is required
			Number:       "", // Invalid: Number is required
		}

		// when
		result, err := svc.Create(ctx, invalidTreeCreate)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.ErrorContains(t, err, "400: validation error")
	})

	t.Run("should return error when fetching TreeCluster fails", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedError := storage.ErrTreeClusterNotFound

		if TestTreeCreate.TreeClusterID == nil {
			t.Fatal("TreeClusterID must not be nil for this test case")
		}

		treeRepo.EXPECT().Create(ctx, mock.Anything).RunAndReturn(
			func(ctx context.Context, fn func(*entities.Tree, storage.TreeRepository) (bool, error)) (*entities.Tree, error) {
				testTree := &entities.Tree{}
				clusterRepo.EXPECT().GetByID(ctx, *TestTreeCreate.TreeClusterID).Return(nil, expectedError)

				success, err := fn(testTree, treeRepo)
				if !success {
					return nil, err
				}
				return testTree, nil
			},
		)

		// When
		result, err := svc.Create(ctx, TestTreeCreate)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error when fetching Sensor fails", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedError := storage.ErrSensorNotFound
		expectedCluster := TestTreeClusters[0]

		if TestTreeCreate.SensorID == nil {
			t.Fatal("SensorID must not be nil for this test case")
		}

		treeRepo.EXPECT().Create(ctx, mock.Anything).RunAndReturn(
			func(ctx context.Context, fn func(*entities.Tree, storage.TreeRepository) (bool, error)) (*entities.Tree, error) {
				testTree := &entities.Tree{}

				clusterRepo.EXPECT().GetByID(ctx, *TestTreeCreate.TreeClusterID).Return(expectedCluster, nil)

				sensorRepo.EXPECT().GetByID(ctx, *TestTreeCreate.SensorID).Return(nil, expectedError)

				success, err := fn(testTree, treeRepo)
				if !success {
					return nil, err
				}
				return testTree, nil
			},
		)

		// when
		result, err := svc.Create(ctx, TestTreeCreate)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error when creating tree fails", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedCluster := TestTreeClusters[0]
		expectedPrevSensorTree := TestTreesList[1]
		expectedSensor := TestSensors[0]
		expectedError := errors.New("tree creation failed")

		if TestTreeCreate.TreeClusterID == nil {
			t.Fatal("TreeClusterID must not be nil for this test case")
		}
		if TestTreeCreate.SensorID == nil {
			t.Fatal("SensorID must not be nil for this test case")
		}

		treeRepo.EXPECT().Create(ctx, mock.Anything).RunAndReturn(
			func(ctx context.Context, fn func(*entities.Tree, storage.TreeRepository) (bool, error)) (*entities.Tree, error) {
				testTree := &entities.Tree{}

				clusterRepo.EXPECT().GetByID(ctx, *TestTreeCreate.TreeClusterID).Return(expectedCluster, nil)
				treeRepo.EXPECT().GetBySensorID(ctx, expectedSensor.ID).Return(expectedPrevSensorTree, nil)
				sensorRepo.EXPECT().GetByID(ctx, *TestTreeCreate.SensorID).Return(expectedSensor, nil)

				success, err := fn(testTree, treeRepo)
				if !success {
					return nil, err
				}

				return nil, expectedError
			},
		)

		// when
		result, err := svc.Create(ctx, TestTreeCreate)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.EqualError(t, err, expectedError.Error())
	})
}

func TestTreeService_Delete(t *testing.T) {
	ctx := context.Background()

	t.Run("should successfully delete a tree", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedTree := TestTreesList[0]
		expectedTree.TreeCluster = TestTreeClusters[0]

		// Mock expectations
		treeRepo.EXPECT().GetByID(ctx, expectedTree.ID).Return(expectedTree, nil)
		treeRepo.EXPECT().Delete(ctx, expectedTree.ID).Return(nil)

		// when
		err := svc.Delete(ctx, expectedTree.ID)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedTree, TestTreesList[0])
	})

	t.Run("should return error if tree not found", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		id := int32(1)
		expectedError := storage.ErrEntityNotFound("not found")

		// Mock expectations
		treeRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedError)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: tree not found")
	})

	t.Run("should return error if tree deletion fails", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedTree := TestTreesList[0]
		expectedTree.TreeCluster = TestTreeClusters[0]
		expectedError := errors.New("deletion failed")

		treeRepo.EXPECT().GetByID(ctx, expectedTree.ID).Return(expectedTree, nil)
		treeRepo.EXPECT().Delete(ctx, expectedTree.ID).Return(expectedError)

		// when
		err := svc.Delete(ctx, expectedTree.ID)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: deletion failed")
	})

	t.Run("should delete a tree without triggering cluster update when tree has no cluster", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedTree := TestTreesList[0]
		expectedTree.TreeCluster = nil // Tree has no cluster

		// Mock expectations
		treeRepo.EXPECT().GetByID(ctx, expectedTree.ID).Return(expectedTree, nil)
		treeRepo.EXPECT().Delete(ctx, expectedTree.ID).Return(nil)

		// Ensure UpdateCluster is not called
		// TODO: check event

		// when
		err := svc.Delete(ctx, expectedTree.ID)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedTree, TestTreesList[0])
	})
}

func TestTreeService_Update(t *testing.T) {
	ctx := context.Background()

	id := int32(1)

	updatedTree := TestTreesList[0]
	updatedTree.Description = TestTreeUpdate.Description
	updatedTree.TreeCluster = TestTreeClusters[1]

	t.Run("should successfully update a tree", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		currentTree := TestTreesList[0]
		treeCluster := TestTreeClusters[0]
		currentTree.TreeCluster = treeCluster
		sensor := TestSensors[0]
		expectedPrevSensorTree := TestTreesList[1]
		currentTree.Sensor = sensor

		if TestTreeUpdate.TreeClusterID == nil {
			t.Fatal("TreeClusterID must not be nil for this test case")
		}
		if TestTreeUpdate.SensorID == nil {
			t.Fatal("SensorID must not be nil for this test case")
		}

		treeRepo.EXPECT().GetByID(ctx, id).Return(currentTree, nil)

		treeRepo.EXPECT().Update(ctx, id, mock.Anything).RunAndReturn(
			func(ctx context.Context, id int32, fn func(*entities.Tree, storage.TreeRepository) (bool, error)) (*entities.Tree, error) {
				testTree := *currentTree

				clusterRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.TreeClusterID).Return(treeCluster, nil)
				treeRepo.EXPECT().GetBySensorID(ctx, sensor.ID).Return(expectedPrevSensorTree, nil)
				sensorRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.SensorID).Return(sensor, nil)

				success, err := fn(&testTree, treeRepo)
				if !success {
					return nil, err
				}
				return updatedTree, nil
			},
		)

		// when
		result, err := svc.Update(ctx, id, TestTreeUpdate)

		// then
		assert.NoError(t, err)
		assert.Equal(t, updatedTree, result)
	})

	t.Run("should return validation error", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		invalidTreeUpdate := &entities.TreeUpdate{
			Latitude:     0,
			Longitude:    0,
			PlantingYear: -2013,
		}

		// when
		result, err := svc.Update(ctx, 1, invalidTreeUpdate)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		//assert.ErrorContains(t, err, "400: validation error")
	})

	t.Run("should return error if tree not found", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedError := storage.ErrEntityNotFound("not found")

		// Mock expectations
		treeRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedError)

		// when
		result, err := svc.Update(ctx, id, TestTreeUpdate)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.EqualError(t, err, "404: tree not found")
	})

	t.Run("should return error if TreeCluster not found", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedError := storage.ErrTreeClusterNotFound

		currentTree := TestTreesList[0]

		treeRepo.EXPECT().GetByID(ctx, id).Return(currentTree, nil)

		treeRepo.EXPECT().Update(ctx, id, mock.Anything).RunAndReturn(
			func(ctx context.Context, id int32, fn func(*entities.Tree, storage.TreeRepository) (bool, error)) (*entities.Tree, error) {
				testTree := *currentTree

				clusterRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.TreeClusterID).Return(nil, expectedError)

				success, err := fn(&testTree, treeRepo)
				if !success {
					return nil, err
				}
				return &testTree, nil
			},
		)

		// when
		result, err := svc.Update(ctx, id, TestTreeUpdate)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error if Sensor not found", func(t *testing.T) {
		// Given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedError := storage.ErrSensorNotFound

		currentTree := TestTreesList[0]
		treeCluster := TestTreeClusters[0]
		currentTree.TreeCluster = treeCluster

		treeRepo.EXPECT().GetByID(ctx, id).Return(currentTree, nil)

		treeRepo.EXPECT().Update(ctx, id, mock.Anything).RunAndReturn(
			func(ctx context.Context, id int32, fn func(*entities.Tree, storage.TreeRepository) (bool, error)) (*entities.Tree, error) {
				testTree := *currentTree

				clusterRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.TreeClusterID).Return(treeCluster, nil)

				sensorRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.SensorID).Return(nil, expectedError)

				success, err := fn(&testTree, treeRepo)
				if !success {
					return nil, err
				}
				return &testTree, nil
			},
		)

		// When
		result, err := svc.Update(ctx, id, TestTreeUpdate)

		// Then
		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error if updating tree fails", func(t *testing.T) {
		// Given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		expectedError := errors.New("update failed")

		currentTree := TestTreesList[0]
		treeCluster := TestTreeClusters[0]
		currentTree.TreeCluster = treeCluster
		sensor := TestSensors[0]
		expectedPrevSensorTree := TestTreesList[1]
		currentTree.Sensor = sensor

		treeRepo.EXPECT().GetByID(ctx, id).Return(currentTree, nil)
		treeRepo.EXPECT().Update(ctx, id, mock.Anything).RunAndReturn(
			func(ctx context.Context, id int32, fn func(*entities.Tree, storage.TreeRepository) (bool, error)) (*entities.Tree, error) {
				testTree := *currentTree

				clusterRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.TreeClusterID).Return(treeCluster, nil)
				treeRepo.EXPECT().GetBySensorID(ctx, sensor.ID).Return(expectedPrevSensorTree, nil)
				sensorRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.SensorID).Return(sensor, nil)

				success, err := fn(&testTree, treeRepo)
				if !success {
					return nil, err
				}
				return nil, expectedError
			},
		)

		// when
		result, err := svc.Update(ctx, id, TestTreeUpdate)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})
}

func TestTreeService_EventSystem(t *testing.T) {
	t.Run("should publish create tree event on create tree", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)

		expectedTree := *TestTreesList[0]
		createTree := &entities.TreeCreate{
			Species:      "Oak",
			Latitude:     testLatitude,
			Longitude:    testLongitude,
			PlantingYear: 2023,
			Number:       "T001",
		}

		// EventSystem
		eventManager := worker.NewEventManager(entities.EventTypeCreateTree)
		expectedEvent := entities.NewEventCreateTree(&expectedTree, nil)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		treeRepo.EXPECT().Create(ctx, mock.Anything).Return(&expectedTree, nil)
		svc := tree.NewTreeService(treeRepo, sensorRepo, treeClusterRepo, eventManager)

		// when
		subID, ch, err := eventManager.Subscribe(entities.EventTypeCreateTree)
		if err != nil {
			t.Fatal("failed to subscribe to event manager")
		}
		_, _ = svc.Create(ctx, createTree)

		// then
		select {
		case recievedEvent := <-ch:
			assert.Equal(t, recievedEvent, expectedEvent)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}

		_ = eventManager.Unsubscribe(entities.EventTypeCreateTree, subID)
	})

	t.Run("should publish update tree event on update tree", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)

		prevTree := *TestTreesList[0]
		expectedTree := *TestTreesList[0]
		expectedTree.TreeCluster = TestTreeClusters[0]

		// Event
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		expectedEvent := entities.NewEventUpdateTree(&prevTree, &expectedTree, nil)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		// Mock expectations
		treeRepo.EXPECT().GetByID(ctx, prevTree.ID).Return(&prevTree, nil)
		treeRepo.EXPECT().Update(ctx, prevTree.ID, mock.Anything).RunAndReturn(
			func(ctx context.Context, id int32, fn func(*entities.Tree, storage.TreeRepository) (bool, error)) (*entities.Tree, error) {
				testTree := prevTree

				clusterRepo.EXPECT().GetByID(ctx, TestTreeClusters[0].ID).Return(TestTreeClusters[0], nil)

				success, err := fn(&testTree, treeRepo)
				if !success {
					return nil, err
				}
				return &expectedTree, nil
			},
		)

		svc := tree.NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		// when
		subID, ch, err := eventManager.Subscribe(entities.EventTypeUpdateTree)
		if err != nil {
			t.Fatal("failed to subscribe to event manager")
		}
		_, _ = svc.Update(ctx, prevTree.ID, &entities.TreeUpdate{
			TreeClusterID: &TestTreeClusters[0].ID,
			SensorID:      nil,
			PlantingYear:  expectedTree.PlantingYear,
			Species:       expectedTree.Species,
			Number:        expectedTree.Number,
			Latitude:      expectedTree.Latitude,
			Longitude:     expectedTree.Longitude,
			Description:   expectedTree.Description,
		})

		// then
		select {
		case recievedEvent := <-ch:
			assert.Equal(t, recievedEvent, expectedEvent)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}

		_ = eventManager.Unsubscribe(entities.EventTypeUpdateTree, subID)
	})

	t.Run("should publish delete tree event on delete tree", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)

		treeToDelete := *TestTreesList[0]

		// EventSystem
		eventManager := worker.NewEventManager(entities.EventTypeDeleteTree)
		expectedEvent := entities.NewEventDeleteTree(&treeToDelete)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		// Mock expectations
		treeRepo.EXPECT().GetByID(ctx, treeToDelete.ID).Return(&treeToDelete, nil)
		treeRepo.EXPECT().Delete(ctx, treeToDelete.ID).Return(nil)

		svc := tree.NewTreeService(treeRepo, sensorRepo, treeClusterRepo, eventManager)

		// when
		subID, ch, err := eventManager.Subscribe(entities.EventTypeDeleteTree)
		if err != nil {
			t.Fatal("failed to subscribe to event manager")
		}
		_ = svc.Delete(ctx, treeToDelete.ID)

		// then
		select {
		case recievedEvent := <-ch:
			assert.Equal(t, recievedEvent, expectedEvent)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}

		_ = eventManager.Unsubscribe(entities.EventTypeDeleteTree, subID)
	})
}

func TestTreeService_UpdateWateringStatuses(t *testing.T) {
	t.Run("should update »just watered« watering status of trees successfully", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)
		svc := tree.NewTreeService(treeRepo, sensorRepo, treeClusterRepo, globalEventManager)

		staleDate := time.Now().Add(-34 * time.Hour)
		recentDate := time.Now().Add(-2 * time.Hour)

		staleTree := &entities.Tree{
			ID:             1,
			LastWatered:    &staleDate, // Older than 24h
			WateringStatus: entities.WateringStatusJustWatered,
		}
		recentTree := &entities.Tree{
			ID:             2,
			LastWatered:    &recentDate,
			WateringStatus: entities.WateringStatusJustWatered,
		}
		expectList := []*entities.Tree{staleTree, recentTree}

		// when
		treeRepo.EXPECT().GetAll(mock.Anything, entities.TreeQuery{}).Return(expectList, int64(len(expectList)), nil)
		treeRepo.EXPECT().Update(mock.Anything, staleTree.ID, mock.Anything).Return(staleTree, nil)

		err := svc.UpdateWateringStatuses(context.Background())

		// then
		assert.NoError(t, err)
		treeRepo.AssertCalled(t, "GetAll", mock.Anything, entities.TreeQuery{})
		treeRepo.AssertCalled(t, "Update", mock.Anything, staleTree.ID, mock.Anything)
		treeRepo.AssertExpectations(t)
	})

	t.Run("should do nothing when there are no trees with correct watering status", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)
		svc := tree.NewTreeService(treeRepo, sensorRepo, treeClusterRepo, globalEventManager)

		recentDate := time.Now().Add(-2 * time.Hour)
		recentTree := &entities.Tree{
			ID:             1,
			LastWatered:    &recentDate,
			WateringStatus: entities.WateringStatusJustWatered,
		}

		expectList := []*entities.Tree{recentTree}

		// when
		treeRepo.EXPECT().GetAll(mock.Anything, entities.TreeQuery{}).Return(expectList, int64(len(expectList)), nil)

		err := svc.UpdateWateringStatuses(context.Background())

		// then
		assert.NoError(t, err)
		treeRepo.AssertCalled(t, "GetAll", mock.Anything, entities.TreeQuery{})
		treeRepo.AssertNotCalled(t, "GetAllLatestSensorDataByClusterID")
		treeRepo.AssertNotCalled(t, "GetBySensorIDs")
		treeRepo.AssertNotCalled(t, "Update")
		treeRepo.AssertExpectations(t)
	})

	t.Run("should return an error when fetching trees fails", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)
		svc := tree.NewTreeService(treeRepo, sensorRepo, treeClusterRepo, globalEventManager)

		// when
		expectedErr := errors.New("database error")
		treeRepo.EXPECT().GetAll(mock.Anything, entities.TreeQuery{}).Return(nil, int64(0), expectedErr)

		err := svc.UpdateWateringStatuses(context.Background())

		// then
		assert.Error(t, err)
		assert.Equal(t, expectedErr, err)
		treeRepo.AssertCalled(t, "GetAll", mock.Anything, entities.TreeQuery{})
		treeRepo.AssertNotCalled(t, "Update")
		treeRepo.AssertExpectations(t)
	})

	t.Run("should log an error when updating a tree fails", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)
		svc := tree.NewTreeService(treeRepo, sensorRepo, treeClusterRepo, globalEventManager)

		staleDate := time.Now().Add(-34 * time.Hour)
		staleTree := &entities.Tree{
			ID:             1,
			LastWatered:    &staleDate, // Older than 24h
			WateringStatus: entities.WateringStatusJustWatered,
		}
		expectList := []*entities.Tree{staleTree}

		// when
		treeRepo.EXPECT().GetAll(mock.Anything, entities.TreeQuery{}).Return(expectList, int64(len(expectList)), nil)
		treeRepo.EXPECT().Update(mock.Anything, staleTree.ID, mock.Anything).Return(nil, errors.New("update failed"))

		err := svc.UpdateWateringStatuses(context.Background())

		// then
		treeRepo.AssertCalled(t, "GetAll", mock.Anything, entities.TreeQuery{})
		treeRepo.AssertCalled(t, "Update", mock.Anything, staleTree.ID, mock.Anything)
		treeRepo.AssertExpectations(t)
		assert.NoError(t, err)
	})
}

func TestTreeService_Ready(t *testing.T) {
	t.Run("should return true when treeRepo and sensorRepo are initialized", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)

		svc := tree.NewTreeService(treeRepo, sensorRepo, nil, nil)

		// when
		result := svc.Ready()

		// then
		assert.True(t, result)
	})

	t.Run("should return false when treeRepo is nil", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)

		svc := tree.NewTreeService(nil, sensorRepo, nil, nil)

		// when
		result := svc.Ready()

		// then
		assert.False(t, result)
	})

	t.Run("should return false when sensorRepo is nil", func(t *testing.T) {
		// given
		treeRepo := storageMock.NewMockTreeRepository(t)

		svc := tree.NewTreeService(treeRepo, nil, nil, nil)

		// when
		result := svc.Ready()

		// then
		assert.False(t, result)
	})

	t.Run("should return false when both treeRepo and sensorRepo are nil", func(t *testing.T) {
		// given
		svc := tree.NewTreeService(nil, nil, nil, nil)

		// when
		result := svc.Ready()

		// then
		assert.False(t, result)
	})
}
