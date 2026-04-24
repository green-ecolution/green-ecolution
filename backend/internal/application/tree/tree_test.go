package tree

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"

	"github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"

	"github.com/stretchr/testify/assert"

	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
)

var (
	globalEventManager = worker.NewEventManager() //tree.EventTypeUpdate, cluster.EventTypeUpdate
	testLatitude       = 9.446741
	testLongitude      = 54.801539
	testCoordinate     = shared.MustNewCoordinate(testLatitude, testLongitude)
	testMapCfg         = config.MapConfig{
		NearestTreeMaxRadius:    500,
		NearestTreeDefaultLimit: 10,
		NearestTreeMaxLimit:     50,
	}
	TestTreeClusters = []*cluster.TreeCluster{
		{
			ID:            1,
			CreatedAt:     time.Now(),
			UpdatedAt:     time.Now(),
			Name:          "Cluster 1",
			Address:       "123 Main St",
			Description:   "Test description",
			SoilCondition: cluster.TreeSoilConditionLehmig,
			Archived:      false,
			Coordinate:    &testCoordinate,
			TreeIDs:       utils.Map(TestTreesList, func(t *tree.Tree) int32 { return t.ID }),
		},
		{
			ID:            2,
			CreatedAt:     time.Now(),
			UpdatedAt:     time.Now(),
			Name:          "Cluster 2",
			Address:       "456 Second St",
			Description:   "Test description",
			SoilCondition: cluster.TreeSoilConditionSandig,
			Archived:      false,
			Coordinate:    nil,
			TreeIDs:       nil,
			LastWatered:   nil,
		},
	}

	TestTreesList = []*tree.Tree{
		{
			ID:             1,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Oak",
			Number:         "T001",
			Coordinate:     shared.MustNewCoordinate(testLatitude, testLongitude),
			Description:    "A mature oak tree",
			PlantingYear:   tree.MustNewPlantingYear(2023),
			WateringStatus: shared.WateringStatusBad,
			LastWatered:    nil,
		},
		{
			ID:             2,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Pine",
			Number:         "T002",
			Coordinate:     shared.MustNewCoordinate(9.446700, 54.801510),
			Description:    "A young pine tree",
			PlantingYear:   tree.MustNewPlantingYear(2023),
			WateringStatus: shared.WateringStatusUnknown,
			LastWatered:    nil,
		},
	}

	testFilterTrees = []*tree.Tree{
		{
			ID:             1,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Oak",
			Number:         "T001",
			Coordinate:     shared.MustNewCoordinate(testLatitude, testLongitude),
			Description:    "A mature oak tree",
			TreeClusterID:  &TestTreeClusters[0].ID,
			WateringStatus: shared.WateringStatusGood,
			PlantingYear:   tree.MustNewPlantingYear(2023),
		},
		{
			ID:             2,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Pine",
			Number:         "T002",
			Coordinate:     shared.MustNewCoordinate(testLatitude, testLongitude),
			TreeClusterID:  &TestTreeClusters[0].ID,
			Description:    "A young pine tree",
			WateringStatus: shared.WateringStatusBad,
			PlantingYear:   tree.MustNewPlantingYear(2022),
		},
	}

	TestSensors = []*sensor.Sensor{
		{
			ID:         sensor.MustNewSensorID("sensor-1"),
			CreatedAt:  time.Now(),
			UpdatedAt:  time.Now(),
			Status:     sensor.SensorStatusUnknown,
			Coordinate: shared.MustNewCoordinate(54.82124518093376, 9.485702120628517),
			LatestData: TestSensorDataBad,
		},
		{
			ID:         sensor.MustNewSensorID("sensor-2"),
			CreatedAt:  time.Now(),
			UpdatedAt:  time.Now(),
			Status:     sensor.SensorStatusUnknown,
			Coordinate: shared.MustNewCoordinate(54.787809938410133, 9.444052105200551),
			LatestData: &sensor.SensorData{},
		},
	}

	TestTreeCreate = &tree.TreeCreate{
		Species:       "Oak",
		Coordinate:    shared.MustNewCoordinate(testLatitude, testLongitude),
		PlantingYear:  tree.MustNewPlantingYear(2023),
		Number:        "T001",
		Description:   "Test tree description",
		TreeClusterID: utils.P(int32(1)),
		SensorID:      utils.P(sensor.MustNewSensorID("sensor-1")),
	}

	TestTreeUpdate = &tree.TreeUpdate{
		TreeClusterID: utils.P(int32(1)),
		SensorID:      utils.P(sensor.MustNewSensorID("sensor-1")),
		PlantingYear:  tree.MustNewPlantingYear(2023),
		Species:       "Oak",
		Number:        "T001",
		Coordinate:    shared.MustNewCoordinate(testLatitude, testLongitude),
		Description:   "Updated description",
	}

	TestSensorDataBad = &sensor.SensorData{
		ID:        1,
		SensorID:  sensor.MustNewSensorID("sensor-1"),
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
		Data: &sensor.MqttPayload{
			Device:      "sensor-1",
			Temperature: 2.0,
			Humidity:    0.5,
			Battery:     3.3,
			Watermarks: []sensor.Watermark{
				{
					Resistance: 2000,
					Centibar:   80,
					Depth:      30,
				},
				{
					Resistance: 2200,
					Centibar:   85,
					Depth:      60,
				},
				{
					Resistance: 2500,
					Centibar:   90,
					Depth:      90,
				},
			},
		},
	}
)

func TestTreeService_GetAll(t *testing.T) {
	ctx := context.Background()

	t.Run("should return all trees when successful", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedTrees := TestTreesList
		treeRepo.EXPECT().GetAll(ctx, tree.TreeQuery{}).Return(expectedTrees, int64(len(expectedTrees)), nil)

		trees, totalCount, err := svc.GetAll(ctx, tree.TreeQuery{})

		assert.NoError(t, err)
		assert.Equal(t, expectedTrees, trees)
		assert.Equal(t, totalCount, int64(len(expectedTrees)))
	})

	t.Run("should return all trees when successful with provider", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedTrees := TestTreesList
		treeRepo.EXPECT().GetAll(ctx, tree.TreeQuery{Query: shared.Query{Provider: "test-provider"}}).Return(expectedTrees, int64(len(expectedTrees)), nil)

		trees, totalCount, err := svc.GetAll(ctx, tree.TreeQuery{Query: shared.Query{Provider: "test-provider"}})

		assert.NoError(t, err)
		assert.Equal(t, expectedTrees, trees)
		assert.Equal(t, totalCount, int64(len(expectedTrees)))
	})

	t.Run("should return empty slice when no trees are found", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		treeRepo.EXPECT().GetAll(ctx, tree.TreeQuery{}).Return([]*tree.Tree{}, int64(0), nil)

		trees, totalCount, err := svc.GetAll(ctx, tree.TreeQuery{})

		assert.NoError(t, err)
		assert.Empty(t, trees)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error when GetAll fails", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedError := errors.New("GetAll failed")
		treeRepo.EXPECT().GetAll(ctx, tree.TreeQuery{}).Return(nil, int64(0), expectedError)

		trees, totalCount, err := svc.GetAll(ctx, tree.TreeQuery{})

		assert.Error(t, err)
		assert.Nil(t, trees)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return trees filtered by watering status and planting years", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedTrees := testFilterTrees
		treeRepo.EXPECT().GetAll(ctx, tree.TreeQuery{
			WateringStatuses: []shared.WateringStatus{shared.WateringStatusGood, shared.WateringStatusBad},
			PlantingYears:    []int32{2022, 2023},
			HasCluster:       utils.P(true),
		}).Return(expectedTrees, int64(len(expectedTrees)), nil)

		trees, totalCount, err := svc.GetAll(ctx, tree.TreeQuery{
			WateringStatuses: []shared.WateringStatus{shared.WateringStatusGood, shared.WateringStatusBad},
			PlantingYears:    []int32{2022, 2023},
			HasCluster:       utils.P(true),
		})

		assert.NoError(t, err)
		assert.Equal(t, expectedTrees, trees)
		assert.Equal(t, totalCount, int64(len(expectedTrees)))
	})
}

func TestTreeService_GetByID(t *testing.T) {
	ctx := context.Background()

	treeRepo := storageMock.NewMockTreeRepository(t)
	sensorRepo := storageMock.NewMockSensorRepository(t)
	clusterRepo := storageMock.NewMockTreeClusterRepository(t)
	eventManager := worker.NewEventManager(tree.EventTypeUpdate)
	svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

	t.Run("should return tree when found", func(t *testing.T) {
		id := int32(1)
		expectedTree := TestTreesList[0]
		treeRepo.EXPECT().GetByID(ctx, id).Return(expectedTree, nil)

		tree, err := svc.GetByID(ctx, id)

		assert.NoError(t, err)
		assert.Equal(t, expectedTree, tree)
	})

	t.Run("should return error if tree not found", func(t *testing.T) {
		id := int32(2)
		expectedError := shared.ErrEntityNotFound("not found")
		treeRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedError)

		tree, err := svc.GetByID(ctx, id)

		assert.Error(t, err)
		assert.Nil(t, tree)
	})

	t.Run("should return error for unexpected repository error", func(t *testing.T) {
		id := int32(3)
		expectedError := errors.New("unexpected error")
		treeRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedError)

		tree, err := svc.GetByID(ctx, id)

		assert.Error(t, err)
		assert.Nil(t, tree)
	})
}

func TestTreeService_GetBySensorID(t *testing.T) {
	ctx := context.Background()

	t.Run("should return tree when found", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		id := sensor.MustNewSensorID("sensor-1")
		expectedTree := TestTreesList[0]
		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{SensorID: &id}).Return([]*tree.Tree{expectedTree}, int64(1), nil)

		result, err := svc.GetBySensorID(ctx, id)

		assert.NoError(t, err)
		assert.Equal(t, expectedTree, result)
	})

	t.Run("should return error if tree not found", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		id := sensor.MustNewSensorID("sensor-2")
		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{SensorID: &id}).Return([]*tree.Tree{}, int64(0), nil)

		result, err := svc.GetBySensorID(ctx, id)

		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error if sensor not found", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		id := sensor.MustNewSensorID("sensor-2")
		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{SensorID: &id}).Return([]*tree.Tree{}, int64(0), nil)

		result, err := svc.GetBySensorID(ctx, id)

		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error for unexpected repository error", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		id := sensor.MustNewSensorID("sensor-3")
		expectedError := errors.New("unexpected error")
		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{SensorID: &id}).Return(nil, int64(0), expectedError)

		result, err := svc.GetBySensorID(ctx, id)

		assert.Error(t, err)
		assert.Nil(t, result)
	})
}

func TestTreeService_Create(t *testing.T) {
	ctx := context.Background()

	t.Run("should successfully create a new tree", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate, tree.EventTypeCreate, tree.EventTypeDelete)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedTree := TestTreesList[0]
		expectedPrevSensorTree := TestTreesList[1]
		expectedCluster := TestTreeClusters[0]
		expectedSensor := TestSensors[0]

		clusterRepo.EXPECT().GetByID(ctx, *TestTreeCreate.TreeClusterID).Return(expectedCluster, nil)
		sensorRepo.EXPECT().GetByID(ctx, *TestTreeCreate.SensorID).Return(expectedSensor, nil)
		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{SensorID: &expectedSensor.ID}).Return([]*tree.Tree{expectedPrevSensorTree}, int64(1), nil)
		treeRepo.EXPECT().Create(ctx, mock.Anything).Return(expectedTree, nil)

		result, err := svc.Create(ctx, TestTreeCreate)

		assert.NoError(t, err)
		assert.Equal(t, expectedTree, result)
	})

	t.Run("should return error when fetching TreeCluster fails", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		clusterRepo.EXPECT().GetByID(ctx, *TestTreeCreate.TreeClusterID).Return(nil, cluster.ErrNotFound)

		result, err := svc.Create(ctx, TestTreeCreate)

		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error when fetching Sensor fails", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedCluster := TestTreeClusters[0]
		clusterRepo.EXPECT().GetByID(ctx, *TestTreeCreate.TreeClusterID).Return(expectedCluster, nil)
		sensorRepo.EXPECT().GetByID(ctx, *TestTreeCreate.SensorID).Return(nil, sensor.ErrNotFound)

		result, err := svc.Create(ctx, TestTreeCreate)

		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error when creating tree fails", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedCluster := TestTreeClusters[0]
		expectedPrevSensorTree := TestTreesList[1]
		expectedSensor := TestSensors[0]
		expectedError := errors.New("tree creation failed")

		clusterRepo.EXPECT().GetByID(ctx, *TestTreeCreate.TreeClusterID).Return(expectedCluster, nil)
		sensorRepo.EXPECT().GetByID(ctx, *TestTreeCreate.SensorID).Return(expectedSensor, nil)
		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{SensorID: &expectedSensor.ID}).Return([]*tree.Tree{expectedPrevSensorTree}, int64(1), nil)
		treeRepo.EXPECT().Create(ctx, mock.Anything).Return(nil, expectedError)

		result, err := svc.Create(ctx, TestTreeCreate)

		assert.Error(t, err)
		assert.Nil(t, result)
		assert.EqualError(t, err, expectedError.Error())
	})
}

func TestTreeService_Delete(t *testing.T) {
	ctx := context.Background()

	t.Run("should successfully delete a tree", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedTree := TestTreesList[0]
		expectedTree.TreeClusterID = &TestTreeClusters[0].ID

		treeRepo.EXPECT().GetByID(ctx, expectedTree.ID).Return(expectedTree, nil)
		treeRepo.EXPECT().Delete(ctx, expectedTree.ID).Return(nil)

		err := svc.Delete(ctx, expectedTree.ID)

		assert.NoError(t, err)
	})

	t.Run("should return error if tree not found", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		id := int32(1)
		expectedError := shared.ErrEntityNotFound("not found")
		treeRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedError)

		err := svc.Delete(ctx, id)

		assert.Error(t, err)
	})

	t.Run("should return error if tree deletion fails", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedTree := TestTreesList[0]
		expectedTree.TreeClusterID = &TestTreeClusters[0].ID
		expectedError := errors.New("deletion failed")

		treeRepo.EXPECT().GetByID(ctx, expectedTree.ID).Return(expectedTree, nil)
		treeRepo.EXPECT().Delete(ctx, expectedTree.ID).Return(expectedError)

		err := svc.Delete(ctx, expectedTree.ID)

		assert.Error(t, err)
	})

	t.Run("should delete a tree without triggering cluster update when tree has no cluster", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedTree := TestTreesList[0]
		expectedTree.TreeClusterID = nil

		treeRepo.EXPECT().GetByID(ctx, expectedTree.ID).Return(expectedTree, nil)
		treeRepo.EXPECT().Delete(ctx, expectedTree.ID).Return(nil)

		err := svc.Delete(ctx, expectedTree.ID)

		assert.NoError(t, err)
	})
}

func TestTreeService_Update(t *testing.T) {
	ctx := context.Background()
	id := int32(1)

	updatedTree := TestTreesList[0]
	updatedTree.Description = TestTreeUpdate.Description
	updatedTree.TreeClusterID = &TestTreeClusters[1].ID

	t.Run("should successfully update a tree", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		currentTree := TestTreesList[0]
		treeCluster := TestTreeClusters[0]
		currentTree.TreeClusterID = &treeCluster.ID
		sensorEntity := TestSensors[0]
		expectedPrevSensorTree := TestTreesList[1]
		currentTree.SensorID = &sensorEntity.ID

		treeRepo.EXPECT().GetByID(ctx, id).Return(currentTree, nil)
		clusterRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.TreeClusterID).Return(treeCluster, nil)
		sensorRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.SensorID).Return(sensorEntity, nil)
		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{SensorID: &sensorEntity.ID}).Return([]*tree.Tree{expectedPrevSensorTree}, int64(1), nil)
		treeRepo.EXPECT().Update(ctx, id, mock.Anything).Return(updatedTree, nil)

		result, err := svc.Update(ctx, id, TestTreeUpdate)

		assert.NoError(t, err)
		assert.Equal(t, updatedTree, result)
	})

	t.Run("should return error if tree not found", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedError := shared.ErrEntityNotFound("not found")
		treeRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedError)

		result, err := svc.Update(ctx, id, TestTreeUpdate)

		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error if TreeCluster not found", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		currentTree := TestTreesList[0]
		treeRepo.EXPECT().GetByID(ctx, id).Return(currentTree, nil)
		clusterRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.TreeClusterID).Return(nil, cluster.ErrNotFound)

		result, err := svc.Update(ctx, id, TestTreeUpdate)

		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error if Sensor not found", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		currentTree := TestTreesList[0]
		treeCluster := TestTreeClusters[0]
		currentTree.TreeClusterID = &treeCluster.ID

		treeRepo.EXPECT().GetByID(ctx, id).Return(currentTree, nil)
		clusterRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.TreeClusterID).Return(treeCluster, nil)
		sensorRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.SensorID).Return(nil, sensor.ErrNotFound)

		result, err := svc.Update(ctx, id, TestTreeUpdate)

		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error if updating tree fails", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedError := errors.New("update failed")
		currentTree := TestTreesList[0]
		treeCluster := TestTreeClusters[0]
		currentTree.TreeClusterID = &treeCluster.ID
		sensorEntity := TestSensors[0]
		expectedPrevSensorTree := TestTreesList[1]
		currentTree.SensorID = &sensorEntity.ID

		treeRepo.EXPECT().GetByID(ctx, id).Return(currentTree, nil)
		clusterRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.TreeClusterID).Return(treeCluster, nil)
		sensorRepo.EXPECT().GetByID(ctx, *TestTreeUpdate.SensorID).Return(sensorEntity, nil)
		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{SensorID: &sensorEntity.ID}).Return([]*tree.Tree{expectedPrevSensorTree}, int64(1), nil)
		treeRepo.EXPECT().Update(ctx, id, mock.Anything).Return(nil, expectedError)

		result, err := svc.Update(ctx, id, TestTreeUpdate)

		assert.Error(t, err)
		assert.Nil(t, result)
	})
}

func TestTreeService_EventSystem(t *testing.T) {
	t.Run("should publish create tree event on create tree", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)

		expectedTree := *TestTreesList[0]
		createTree := &tree.TreeCreate{
			Species:      "Oak",
			Coordinate:   shared.MustNewCoordinate(testLatitude, testLongitude),
			PlantingYear: tree.MustNewPlantingYear(2023),
			Number:       "T001",
		}

		eventManager := worker.NewEventManager(tree.EventTypeCreate)
		expectedEvent := tree.NewEventCreate(&expectedTree, nil)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		treeRepo.EXPECT().Create(ctx, mock.Anything).Return(&expectedTree, nil)
		svc := NewTreeService(treeRepo, sensorRepo, treeClusterRepo, eventManager, testMapCfg)

		subID, ch, err := eventManager.Subscribe(tree.EventTypeCreate)
		if err != nil {
			t.Fatal("failed to subscribe to event manager")
		}
		_, _ = svc.Create(ctx, createTree)

		select {
		case recievedEvent := <-ch:
			assert.Equal(t, recievedEvent, expectedEvent)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}

		_ = eventManager.Unsubscribe(tree.EventTypeCreate, subID)
	})

	t.Run("should publish update tree event on update tree", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)

		prevTree := *TestTreesList[0]
		expectedTree := *TestTreesList[0]
		expectedTree.TreeClusterID = &TestTreeClusters[0].ID

		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		expectedEvent := tree.NewEventUpdate(&prevTree, &expectedTree, nil)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		treeRepo.EXPECT().GetByID(ctx, prevTree.ID).Return(&prevTree, nil)
		clusterRepo.EXPECT().GetByID(ctx, TestTreeClusters[0].ID).Return(TestTreeClusters[0], nil)
		treeRepo.EXPECT().Update(ctx, prevTree.ID, mock.Anything).Return(&expectedTree, nil)

		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		subID, ch, err := eventManager.Subscribe(tree.EventTypeUpdate)
		if err != nil {
			t.Fatal("failed to subscribe to event manager")
		}
		_, _ = svc.Update(ctx, prevTree.ID, &tree.TreeUpdate{
			TreeClusterID: &TestTreeClusters[0].ID,
			SensorID:      nil,
			PlantingYear:  expectedTree.PlantingYear,
			Species:       expectedTree.Species,
			Number:        expectedTree.Number,
			Coordinate:    expectedTree.Coordinate,
			Description:   expectedTree.Description,
		})

		select {
		case recievedEvent := <-ch:
			assert.Equal(t, recievedEvent, expectedEvent)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}

		_ = eventManager.Unsubscribe(tree.EventTypeUpdate, subID)
	})

	t.Run("should publish delete tree event on delete tree", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)

		treeToDelete := *TestTreesList[0]

		eventManager := worker.NewEventManager(tree.EventTypeDelete)
		expectedEvent := tree.NewEventDelete(&treeToDelete)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		treeRepo.EXPECT().GetByID(ctx, treeToDelete.ID).Return(&treeToDelete, nil)
		treeRepo.EXPECT().Delete(ctx, treeToDelete.ID).Return(nil)

		svc := NewTreeService(treeRepo, sensorRepo, treeClusterRepo, eventManager, testMapCfg)

		subID, ch, err := eventManager.Subscribe(tree.EventTypeDelete)
		if err != nil {
			t.Fatal("failed to subscribe to event manager")
		}
		_ = svc.Delete(ctx, treeToDelete.ID)

		select {
		case recievedEvent := <-ch:
			assert.Equal(t, recievedEvent, expectedEvent)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}

		_ = eventManager.Unsubscribe(tree.EventTypeDelete, subID)
	})
}

func TestTreeService_UpdateWateringStatuses(t *testing.T) {
	t.Run("should update watering status of trees successfully", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)
		svc := NewTreeService(treeRepo, sensorRepo, treeClusterRepo, globalEventManager, testMapCfg)

		staleDate := time.Now().Add(-34 * time.Hour)
		recentDate := time.Now().Add(-2 * time.Hour)

		sensorWithData := &sensor.Sensor{
			ID: sensor.MustNewSensorID("sensor-1"),
			LatestData: &sensor.SensorData{
				Data: &sensor.MqttPayload{
					Watermarks: []sensor.Watermark{
						{Centibar: 10, Depth: 30},
						{Centibar: 10, Depth: 60},
						{Centibar: 10, Depth: 90},
					},
				},
			},
		}
		staleTree := &tree.Tree{
			ID:             1,
			LastWatered:    &staleDate,
			WateringStatus: shared.WateringStatusJustWatered,
			PlantingYear:   tree.MustNewPlantingYear(int32(time.Now().Year())),
			SensorID:       &sensorWithData.ID,
		}
		recentTree := &tree.Tree{
			ID:             2,
			LastWatered:    &recentDate,
			WateringStatus: shared.WateringStatusJustWatered,
		}
		expectList := []*tree.Tree{staleTree, recentTree}

		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{}).Return(expectList, int64(len(expectList)), nil)
		sensorRepo.EXPECT().GetByID(mock.Anything, sensorWithData.ID).Return(sensorWithData, nil)
		treeRepo.EXPECT().Update(mock.Anything, staleTree.ID, mock.Anything).Return(staleTree, nil)

		err := svc.UpdateWateringStatuses(context.Background())

		assert.NoError(t, err)
	})

	t.Run("should do nothing when there are no trees with correct watering status", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)
		svc := NewTreeService(treeRepo, sensorRepo, treeClusterRepo, globalEventManager, testMapCfg)

		recentDate := time.Now().Add(-2 * time.Hour)
		recentTree := &tree.Tree{
			ID:             1,
			LastWatered:    &recentDate,
			WateringStatus: shared.WateringStatusJustWatered,
		}
		expectList := []*tree.Tree{recentTree}

		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{}).Return(expectList, int64(len(expectList)), nil)

		err := svc.UpdateWateringStatuses(context.Background())

		assert.NoError(t, err)
		treeRepo.AssertNotCalled(t, "Update")
	})

	t.Run("should return an error when fetching trees fails", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)
		svc := NewTreeService(treeRepo, sensorRepo, treeClusterRepo, globalEventManager, testMapCfg)

		expectedErr := errors.New("database error")
		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{}).Return(nil, int64(0), expectedErr)

		err := svc.UpdateWateringStatuses(context.Background())

		assert.Error(t, err)
		assert.Equal(t, expectedErr, err)
		treeRepo.AssertNotCalled(t, "Update")
	})

	t.Run("should log an error when updating a tree fails", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeClusterRepo := storageMock.NewMockTreeClusterRepository(t)
		svc := NewTreeService(treeRepo, sensorRepo, treeClusterRepo, globalEventManager, testMapCfg)

		staleDate := time.Now().Add(-34 * time.Hour)
		sensorWithData := &sensor.Sensor{
			ID: sensor.MustNewSensorID("sensor-1"),
			LatestData: &sensor.SensorData{
				Data: &sensor.MqttPayload{
					Watermarks: []sensor.Watermark{
						{Centibar: 10, Depth: 30},
						{Centibar: 10, Depth: 60},
						{Centibar: 10, Depth: 90},
					},
				},
			},
		}
		staleTree := &tree.Tree{
			ID:             1,
			LastWatered:    &staleDate,
			WateringStatus: shared.WateringStatusJustWatered,
			PlantingYear:   tree.MustNewPlantingYear(int32(time.Now().Year())),
			SensorID:       &sensorWithData.ID,
		}
		expectList := []*tree.Tree{staleTree}

		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{}).Return(expectList, int64(len(expectList)), nil)
		sensorRepo.EXPECT().GetByID(mock.Anything, sensorWithData.ID).Return(sensorWithData, nil)
		treeRepo.EXPECT().Update(mock.Anything, staleTree.ID, mock.Anything).Return(nil, errors.New("update failed"))

		err := svc.UpdateWateringStatuses(context.Background())

		assert.Error(t, err)
	})
}

func TestTreeService_Ready(t *testing.T) {
	t.Run("should return true when treeRepo and sensorRepo are initialized", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		svc := NewTreeService(treeRepo, sensorRepo, nil, nil, testMapCfg)

		result := svc.Ready()

		assert.True(t, result)
	})

	t.Run("should return false when treeRepo is nil", func(t *testing.T) {
		sensorRepo := storageMock.NewMockSensorRepository(t)
		svc := NewTreeService(nil, sensorRepo, nil, nil, testMapCfg)

		result := svc.Ready()

		assert.False(t, result)
	})

	t.Run("should return false when sensorRepo is nil", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewTreeService(treeRepo, nil, nil, nil, testMapCfg)

		result := svc.Ready()

		assert.False(t, result)
	})

	t.Run("should return false when both treeRepo and sensorRepo are nil", func(t *testing.T) {
		svc := NewTreeService(nil, nil, nil, nil, testMapCfg)

		result := svc.Ready()

		assert.False(t, result)
	})
}

func TestTreeService_GetPlantingYears(t *testing.T) {
	ctx := context.Background()

	t.Run("should return planting years successfully", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedYears := []int32{2022, 2023, 2024, 2025}
		treeRepo.EXPECT().GetDistinctPlantingYears(ctx).Return(expectedYears, nil)

		years, err := svc.GetPlantingYears(ctx)

		assert.NoError(t, err)
		assert.Equal(t, expectedYears, years)
	})

	t.Run("should return empty slice when no planting years exist", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		treeRepo.EXPECT().GetDistinctPlantingYears(ctx).Return([]int32{}, nil)

		years, err := svc.GetPlantingYears(ctx)

		assert.NoError(t, err)
		assert.Empty(t, years)
	})

	t.Run("should return error when repository fails", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		expectedError := errors.New("database error")
		treeRepo.EXPECT().GetDistinctPlantingYears(ctx).Return(nil, expectedError)

		years, err := svc.GetPlantingYears(ctx)

		assert.Error(t, err)
		assert.Nil(t, years)
	})
}

func TestTreeService_GetNearestTrees(t *testing.T) {
	ctx := context.Background()
	coord := shared.MustNewCoordinate(54.801539, 9.446741)

	newSvc := func(t *testing.T) (*storageMock.MockTreeRepository, *TreeService) {
		t.Helper()
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg).(*TreeService)
		return treeRepo, svc
	}

	t.Run("should return list sorted by distance on success", func(t *testing.T) {
		treeRepo, svc := newSvc(t)
		expected := []*tree.TreeWithDistance{
			{Tree: TestTreesList[0], Distance: shared.MustNewDistance(5.2)},
			{Tree: TestTreesList[1], Distance: shared.MustNewDistance(42.8)},
		}
		treeRepo.EXPECT().FindNearestTrees(ctx, coord, testMapCfg.NearestTreeMaxRadius, int32(5)).Return(expected, nil)

		got, err := svc.GetNearestTrees(ctx, coord, 5)

		assert.NoError(t, err)
		assert.Equal(t, expected, got)
	})

	t.Run("should use default limit when limit is zero", func(t *testing.T) {
		treeRepo, svc := newSvc(t)
		treeRepo.EXPECT().FindNearestTrees(ctx, coord, testMapCfg.NearestTreeMaxRadius, int32(testMapCfg.NearestTreeDefaultLimit)).
			Return([]*tree.TreeWithDistance{}, nil)

		got, err := svc.GetNearestTrees(ctx, coord, 0)

		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should use default limit when limit is negative", func(t *testing.T) {
		treeRepo, svc := newSvc(t)
		treeRepo.EXPECT().FindNearestTrees(ctx, coord, testMapCfg.NearestTreeMaxRadius, int32(testMapCfg.NearestTreeDefaultLimit)).
			Return([]*tree.TreeWithDistance{}, nil)

		got, err := svc.GetNearestTrees(ctx, coord, -5)

		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should clamp limit to max limit", func(t *testing.T) {
		treeRepo, svc := newSvc(t)
		treeRepo.EXPECT().FindNearestTrees(ctx, coord, testMapCfg.NearestTreeMaxRadius, int32(testMapCfg.NearestTreeMaxLimit)).
			Return([]*tree.TreeWithDistance{}, nil)

		got, err := svc.GetNearestTrees(ctx, coord, 1000)

		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should propagate repository error", func(t *testing.T) {
		treeRepo, svc := newSvc(t)
		treeRepo.EXPECT().FindNearestTrees(ctx, coord, testMapCfg.NearestTreeMaxRadius, int32(testMapCfg.NearestTreeDefaultLimit)).
			Return(nil, errors.New("db down"))

		got, err := svc.GetNearestTrees(ctx, coord, 0)

		assert.Error(t, err)
		assert.Nil(t, got)
	})
}
