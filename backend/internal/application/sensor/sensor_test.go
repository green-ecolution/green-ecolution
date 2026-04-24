package sensor

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
)

var (
	TestListMQTTPayload = []*sensor.MqttPayload{
		{
			Device:      "sensor001",
			Battery:     45.3,
			Humidity:    0.75,
			Temperature: 22.5,
			Latitude:    37.7749,
			Longitude:   -122.4194,
			Watermarks: []sensor.Watermark{
				{Centibar: 30, Resistance: 20, Depth: 30},
				{Centibar: 40, Resistance: 25, Depth: 60},
				{Centibar: 50, Resistance: 30, Depth: 90},
			},
		},
		{
			Device:      "sensor002",
			Battery:     78.9,
			Humidity:    0.60,
			Temperature: 18.3,
			Latitude:    48.8566,
			Longitude:   2.3522,
			Watermarks: []sensor.Watermark{
				{Centibar: 25, Resistance: 18, Depth: 30},
				{Centibar: 35, Resistance: 22, Depth: 60},
				{Centibar: 45, Resistance: 27, Depth: 90},
			},
		},
		{
			Device:      "sensor003",
			Battery:     32.1,
			Humidity:    0.85,
			Temperature: 28.0,
			Latitude:    -33.8688,
			Longitude:   151.2093,
			Watermarks: []sensor.Watermark{
				{Centibar: 20, Resistance: 15, Depth: 30},
				{Centibar: 30, Resistance: 20, Depth: 60},
				{Centibar: 40, Resistance: 25, Depth: 90},
			},
		},
	}

	TestMQTTPayLoadInvalidLong = &sensor.MqttPayload{
		Device:      "sensor001",
		Battery:     45.3,
		Humidity:    0.75,
		Temperature: 22.5,
		Latitude:    37.7749,
		Longitude:   181.0, // invalid
		Watermarks: []sensor.Watermark{
			{Centibar: 30, Resistance: 20, Depth: 30},
			{Centibar: 40, Resistance: 25, Depth: 60},
			{Centibar: 50, Resistance: 30, Depth: 90},
		},
	}

	TestMQTTPayLoadInvalidLat = &sensor.MqttPayload{
		Device:      "sensor001",
		Battery:     45.3,
		Humidity:    0.75,
		Temperature: 22.5,
		Latitude:    91.0, // invalid
		Longitude:   -122.4194,
		Watermarks: []sensor.Watermark{
			{Centibar: 30, Resistance: 20, Depth: 30},
			{Centibar: 40, Resistance: 25, Depth: 60},
			{Centibar: 50, Resistance: 30, Depth: 90},
		},
	}

	TestSensor = &sensor.Sensor{
		ID:         sensor.MustNewSensorID("sensor001"),
		CreatedAt:  time.Now(),
		UpdatedAt:  time.Now(),
		Coordinate: shared.MustNewCoordinate(54.82124518093376, 9.485702120628517),
		Status:     sensor.SensorStatusOnline,
		LatestData: TestSensorData[0],
	}

	TestSensorData = []*sensor.SensorData{
		{
			ID:        1,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
			Data:      TestListMQTTPayload[0],
		},
		{
			ID:        2,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
			Data:      TestListMQTTPayload[1],
		},
		{
			ID:        3,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
			Data:      TestListMQTTPayload[2],
		},
	}

	TestSensorList = []*sensor.Sensor{
		TestSensor,
		{
			ID:         sensor.MustNewSensorID("sensor-2"),
			CreatedAt:  time.Now(),
			UpdatedAt:  time.Now(),
			Coordinate: shared.MustNewCoordinate(54.78780993841013, 9.444052105200551),
			Status:     sensor.SensorStatusOffline,
			LatestData: &sensor.SensorData{},
		},
		{
			ID:         sensor.MustNewSensorID("sensor-3"),
			CreatedAt:  time.Now(),
			UpdatedAt:  time.Now(),
			Coordinate: shared.MustNewCoordinate(54.77933725347423, 9.426465409018832),
			Status:     sensor.SensorStatusUnknown,
			LatestData: &sensor.SensorData{},
		},
		{
			ID:         sensor.MustNewSensorID("sensor-4"),
			CreatedAt:  time.Now(),
			UpdatedAt:  time.Now(),
			Coordinate: shared.MustNewCoordinate(54.82078826498143, 9.489684366114483),
			Status:     sensor.SensorStatusOnline,
			LatestData: &sensor.SensorData{},
		},
	}

	TestSensorNearestTree = &sensor.Sensor{
		ID:         sensor.MustNewSensorID("sensor-05"),
		CreatedAt:  time.Now(),
		UpdatedAt:  time.Now(),
		Coordinate: shared.MustNewCoordinate(54.821535, 9.487200),
		Status:     sensor.SensorStatusOnline,
		LatestData: TestSensorData[0],
	}

	TestNearestTree = &tree.Tree{
		ID:           5,
		CreatedAt:    time.Now(),
		UpdatedAt:    time.Now(),
		Species:      "Oak",
		Number:       "T001",
		Coordinate:   shared.MustNewCoordinate(54.8215076622281, 9.487153277881877),
		Description:  "A mature oak tree",
		PlantingYear: tree.MustNewPlantingYear(2023),
	}
)

func TestSensorService_GetAll(t *testing.T) {
	t.Run("should return all sensors", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		sensorRepo.EXPECT().GetAll(context.Background(), shared.Query{}).Return(TestSensorList, int64(len(TestSensorList)), nil)
		sensors, totalCount, err := svc.GetAll(context.Background(), shared.Query{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, totalCount, int64(len(TestSensorList)))
		assert.Equal(t, TestSensorList, sensors)
	})

	t.Run("should return all sensors by provider", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		sensorRepo.EXPECT().GetAll(context.Background(), shared.Query{Provider: "test-provider"}).Return(TestSensorList, int64(len(TestSensorList)), nil)
		sensors, totalCount, err := svc.GetAll(context.Background(), shared.Query{Provider: "test-provider"})

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestSensorList, sensors)
		assert.Equal(t, totalCount, int64(len(TestSensorList)))
	})

	t.Run("should return error when repository fails", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		sensorRepo.EXPECT().GetAll(context.Background(), shared.Query{}).Return(nil, int64(0), sensor.ErrNotFound)
		sensors, totalCount, err := svc.GetAll(context.Background(), shared.Query{})

		// then
		assert.Error(t, err)
		assert.Nil(t, sensors)
		assert.Equal(t, totalCount, int64(0))
		// assert.EqualError(t, err, "500: sensor not found")
	})
}

func TestSensorService_GetAllDataByID(t *testing.T) {
	t.Run("should return all sensor data", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		sensorRepo.EXPECT().GetAllDataByID(context.Background(), sensor.MustNewSensorID("sensor-1")).Return(TestSensorData, nil)
		sensorData, err := svc.GetAllDataByID(context.Background(), sensor.MustNewSensorID("sensor-1"))

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestSensorData, sensorData)
	})

	t.Run("should return error when no sensor is found", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		sensorRepo.EXPECT().GetAllDataByID(context.Background(), sensor.MustNewSensorID("sensor-1")).Return(nil, sensor.ErrNotFound)
		sensorData, err := svc.GetAllDataByID(context.Background(), sensor.MustNewSensorID("sensor-1"))

		// then
		assert.Error(t, err)
		assert.Nil(t, sensorData)
		// assert.EqualError(t, err, "500: sensor not found")
	})
}

func TestSensorService_GetByID(t *testing.T) {
	t.Run("should return sensor when found", func(t *testing.T) {
		// given
		id := sensor.MustNewSensorID("sensor-1")
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		sensorRepo.EXPECT().GetByID(context.Background(), id).Return(TestSensor, nil)

		// when
		sensor, err := svc.GetByID(context.Background(), id)

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestSensor, sensor)
	})

	t.Run("should return error if sensor not found", func(t *testing.T) {
		// given
		id := sensor.MustNewSensorID("sensor-1")
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		expectedErr := shared.ErrEntityNotFound("not found")
		sensorRepo.EXPECT().GetByID(context.Background(), id).Return(nil, expectedErr)

		// when
		sensor, err := svc.GetByID(context.Background(), id)

		// then
		assert.Error(t, err)
		assert.Nil(t, sensor)
		// assert.EqualError(t, err, "404: sensor not found")
	})
}

func TestSensorService_Create(t *testing.T) {
	newSensor := &sensor.SensorCreate{
		ID:         sensor.MustNewSensorID("sensor-1"),
		Status:     sensor.SensorStatusOnline,
		LatestData: TestSensor.LatestData,
		Coordinate: shared.MustNewCoordinate(9.446741, 54.801539),
	}

	t.Run("should successfully create a new sensor", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		sensorRepo.EXPECT().Create(context.Background(), mock.Anything).Return(TestSensor, nil)

		// when
		result, err := svc.Create(context.Background(), newSensor)

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestSensor, result)
	})

	t.Run("should successfully create a new sensor without data", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		newSensor.LatestData = &sensor.SensorData{}

		sensorRepo.EXPECT().Create(context.Background(), mock.Anything).Return(TestSensor, nil)

		// when
		result, err := svc.Create(context.Background(), newSensor)

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestSensor, result)
	})

	t.Run("should return validation error on no status", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		newSensor.Status = ""

		// when
		result, err := svc.Create(context.Background(), newSensor)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error on invalid sensor id", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		newSensor.Status = sensor.SensorStatusOffline
		newSensor.ID = sensor.SensorID{}

		// when
		result, err := svc.Create(context.Background(), newSensor)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return validation error on invalid latitude and longitude", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		newSensor.ID = sensor.MustNewSensorID("sensor-23")
		newSensor.Status = sensor.SensorStatusOffline
		newSensor.Coordinate = shared.Coordinate{}

		// when
		result, err := svc.Create(context.Background(), newSensor)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})

	t.Run("should return an error when creating sensor fails", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		expectedErr := errors.New("Failed to create sensor")

		newSensor.ID = sensor.MustNewSensorID("sensor-23")
		newSensor.Status = sensor.SensorStatusOffline
		newSensor.Coordinate = shared.MustNewCoordinate(9.446741, 54.801539)

		sensorRepo.EXPECT().Create(context.Background(), mock.Anything).Return(nil, expectedErr)

		// when
		result, err := svc.Create(context.Background(), newSensor)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: Failed to create sensor")
	})
}

func TestSensorService_Update(t *testing.T) {
	updateSensor := &sensor.SensorUpdate{
		Status:     sensor.SensorStatusOnline,
		LatestData: TestSensor.LatestData,
		Coordinate: shared.MustNewCoordinate(9.446741, 54.801539),
	}

	t.Run("should successfully update a sensor", func(t *testing.T) {
		// given
		id := sensor.MustNewSensorID("sensor-1")
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		sensorRepo.EXPECT().GetByID(context.Background(), id).Return(TestSensor, nil)

		sensorRepo.EXPECT().Update(context.Background(), id, mock.Anything).Return(TestSensor, nil)

		// when
		result, err := svc.Update(context.Background(), id, updateSensor)

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestSensor, result)
	})

	t.Run("should return an error when sensor ID does not exist", func(t *testing.T) {
		// given
		id := sensor.MustNewSensorID("notFoundID")
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		expectedErr := errors.New("failed to update cluster")

		sensorRepo.EXPECT().GetByID(context.Background(), id).Return(nil, expectedErr)

		// when
		result, err := svc.Update(context.Background(), id, updateSensor)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to update cluster")
	})

	t.Run("should return an error when the update fails", func(t *testing.T) {
		// given
		id := sensor.MustNewSensorID("sensor-1")
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		expectedErr := errors.New("failed to update cluster")

		sensorRepo.EXPECT().GetByID(context.Background(), id).Return(TestSensor, nil)

		sensorRepo.EXPECT().Update(context.Background(), id, mock.Anything).Return(nil, expectedErr)

		// when
		result, err := svc.Update(context.Background(), id, updateSensor)

		// then
		assert.Nil(t, result)
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to update cluster")
	})

	t.Run("should return validation error on invalid latitude and longitude", func(t *testing.T) {
		// given
		id := sensor.MustNewSensorID("sensor-1")
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		updateSensor.Coordinate = shared.Coordinate{}

		// when
		result, err := svc.Update(context.Background(), id, updateSensor)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "validation error")
	})
}

func TestSensorService_Delete(t *testing.T) {
	ctx := context.Background()

	t.Run("should successfully delete a sensor", func(t *testing.T) {
		// given
		id := sensor.MustNewSensorID("sensor-1")
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		sensorRepo.EXPECT().GetByID(ctx, id).Return(TestSensor, nil)
		treeRepo.EXPECT().UnlinkSensorID(ctx, id).Return(nil)
		sensorRepo.EXPECT().Delete(ctx, id).Return(nil)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.NoError(t, err)
	})

	t.Run("should return error if sensor not found", func(t *testing.T) {
		// given
		id := sensor.MustNewSensorID("sensor-1")
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		expectedErr := shared.ErrEntityNotFound("not found")
		sensorRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedErr)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: sensor not found")
	})

	t.Run("should return error if unlinking sensor ID on tree fails", func(t *testing.T) {
		// given
		id := sensor.MustNewSensorID("sensor-1")
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		expectedErr := errors.New("failed to unlink")

		sensorRepo.EXPECT().GetByID(ctx, id).Return(TestSensor, nil)
		treeRepo.EXPECT().UnlinkSensorID(ctx, id).Return(expectedErr)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to unlink")
	})

	t.Run("should return error if deleting sensor fails", func(t *testing.T) {
		// given
		id := sensor.MustNewSensorID("sensor-1")
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		expectedErr := errors.New("failed to delete")

		sensorRepo.EXPECT().GetByID(ctx, id).Return(TestSensor, nil)
		treeRepo.EXPECT().UnlinkSensorID(ctx, id).Return(nil)
		sensorRepo.EXPECT().Delete(ctx, id).Return(expectedErr)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to delete")
	})
}

func TestSensorService_MapSensorToTree(t *testing.T) {
	t.Run("should successfully map sensor to the nearest tree", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testSensor := TestSensorNearestTree
		testTree := TestNearestTree

		treeRepo.EXPECT().
			FindNearestTree(context.Background(), mock.Anything).
			Return(testTree, nil)

		treeRepo.EXPECT().
			Update(context.Background(), testTree.ID, mock.Anything).
			Return(testTree, nil)

		// when
		err := svc.MapSensorToTree(context.Background(), testSensor)

		// then
		assert.NoError(t, err)
	})

	t.Run("should return error if sensor is nil", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		err := svc.MapSensorToTree(context.Background(), nil)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "sensor cannot be nil")
	})

	t.Run("should return error if nearest tree is not found", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testSensor := TestSensorNearestTree

		treeRepo.EXPECT().
			FindNearestTree(context.Background(), testSensor.Coordinate).
			Return(nil, errors.New("tree not found"))

		// when
		err := svc.MapSensorToTree(context.Background(), testSensor)

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "tree not found")
	})

	t.Run("should return error if updating tree with sensor fails", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testSensor := TestSensorNearestTree
		testTree := TestNearestTree

		treeRepo.EXPECT().
			FindNearestTree(context.Background(), mock.Anything).
			Return(testTree, nil)

		treeRepo.EXPECT().
			Update(context.Background(), testTree.ID, mock.Anything).
			Return(nil, errors.New("update failed"))

		// when
		err := svc.MapSensorToTree(context.Background(), testSensor)

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "update failed")
	})
}

func TestReady(t *testing.T) {
	t.Run("should return true if the service is ready", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		ready := svc.Ready()

		// then
		assert.True(t, ready)
	})

	t.Run("should return false if the service is not ready", func(t *testing.T) {
		// give
		svc := NewSensorService(nil, nil, globalEventManager)

		// when
		ready := svc.Ready()

		// then
		assert.False(t, ready)
	})
}

func TestSensorService_UpdateStatuses(t *testing.T) {
	t.Run("should update stale sensors successfully", func(t *testing.T) {
		// given
		ctx := context.Background()
		repo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(repo, treeRepo, globalEventManager)

		staleSensor := &sensor.Sensor{
			ID: sensor.MustNewSensorID("sensor-1"),
		}
		recentSensor := &sensor.Sensor{
			ID: sensor.MustNewSensorID("sensor-2"),
		}
		staleSensorData := &sensor.SensorData{
			CreatedAt: time.Now().Add(-73 * time.Hour), // Older than 72h
		}
		recentSensorData := &sensor.SensorData{
			CreatedAt: time.Now().Add(-1 * time.Hour), // 1 hour ago (not stale)
		}

		expectList := []*sensor.Sensor{staleSensor, recentSensor}

		// when
		repo.EXPECT().GetAll(mock.Anything, shared.Query{}).Return(expectList, int64(len(expectList)), nil)
		repo.EXPECT().GetLatestSensorDataBySensorID(mock.Anything, staleSensor.ID).Return(staleSensorData, nil)
		repo.EXPECT().GetLatestSensorDataBySensorID(mock.Anything, recentSensor.ID).Return(recentSensorData, nil)
		repo.EXPECT().Update(mock.Anything, staleSensor.ID, mock.Anything).Return(staleSensor, nil)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.NoError(t, err)
		repo.AssertCalled(t, "GetAll", mock.Anything, shared.Query{})
		repo.AssertCalled(t, "GetLatestSensorDataBySensorID", mock.Anything, staleSensor.ID)
		repo.AssertCalled(t, "GetLatestSensorDataBySensorID", mock.Anything, recentSensor.ID)
		repo.AssertCalled(t, "Update", mock.Anything, staleSensor.ID, mock.Anything)
		repo.AssertExpectations(t) // Verifies all expectations are met
	})

	t.Run("should do nothing when there are no stale sensors", func(t *testing.T) {
		// given
		ctx := context.Background()
		repo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(repo, treeRepo, globalEventManager)

		freshSensor := &sensor.Sensor{ID: sensor.MustNewSensorID("sensor-1")}
		freshSensorData := &sensor.SensorData{
			CreatedAt: time.Now(),
		}

		expectList := []*sensor.Sensor{freshSensor}

		// when
		repo.EXPECT().GetAll(mock.Anything, shared.Query{}).Return(expectList, int64(len(expectList)), nil)
		repo.EXPECT().GetLatestSensorDataBySensorID(mock.Anything, freshSensor.ID).Return(freshSensorData, nil)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.NoError(t, err)
		repo.AssertCalled(t, "GetAll", mock.Anything, shared.Query{})
		repo.AssertCalled(t, "GetLatestSensorDataBySensorID", mock.Anything, freshSensor.ID)
		repo.AssertNotCalled(t, "Update")
		repo.AssertExpectations(t)
	})

	t.Run("should return an error when fetching sensors fails", func(t *testing.T) {
		// given
		ctx := context.Background()
		repo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(repo, treeRepo, globalEventManager)

		// when
		expectedErr := errors.New("database error")
		repo.EXPECT().GetAll(mock.Anything, shared.Query{}).Return(nil, int64(0), expectedErr)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.Error(t, err)
		assert.Equal(t, expectedErr, err)
		repo.AssertCalled(t, "GetAll", mock.Anything, shared.Query{})
		repo.AssertNotCalled(t, "GetLatestSensorDataBySensorID")
		repo.AssertNotCalled(t, "Update")
		repo.AssertExpectations(t)
	})

	t.Run("should log an error when fetching latest sensor data fails", func(t *testing.T) {
		// given
		ctx := context.Background()
		repo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(repo, treeRepo, globalEventManager)

		staleSensor := &sensor.Sensor{ID: sensor.MustNewSensorID("sensor-1")}
		expectList := []*sensor.Sensor{staleSensor}

		expectedErr := errors.New("failed to fetch sensor data")

		// when
		repo.EXPECT().GetAll(mock.Anything, shared.Query{}).Return(expectList, int64(len(expectList)), nil)
		repo.EXPECT().GetLatestSensorDataBySensorID(mock.Anything, staleSensor.ID).Return(nil, expectedErr)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.NoError(t, err)
		repo.AssertCalled(t, "GetAll", mock.Anything, shared.Query{})
		repo.AssertCalled(t, "GetLatestSensorDataBySensorID", mock.Anything, staleSensor.ID)
		repo.AssertNotCalled(t, "Update")
		repo.AssertExpectations(t)
	})

	t.Run("should log an error when updating a sensor fails", func(t *testing.T) {
		// given
		ctx := context.Background()
		repo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(repo, treeRepo, globalEventManager)

		staleSensor := &sensor.Sensor{ID: sensor.MustNewSensorID("sensor-1")}
		staleSensorData := &sensor.SensorData{
			CreatedAt: time.Now().Add(-100 * time.Hour),
		}

		expectList := []*sensor.Sensor{staleSensor}

		// when
		repo.EXPECT().GetAll(mock.Anything, shared.Query{}).Return(expectList, int64(len(expectList)), nil)
		repo.EXPECT().GetLatestSensorDataBySensorID(mock.Anything, staleSensor.ID).Return(staleSensorData, nil)
		repo.EXPECT().Update(mock.Anything, staleSensor.ID, mock.Anything).Return(nil, errors.New("update failed"))

		err := svc.UpdateStatuses(ctx)

		// then
		repo.AssertCalled(t, "GetAll", mock.Anything, shared.Query{})
		repo.AssertCalled(t, "GetLatestSensorDataBySensorID", mock.Anything, staleSensor.ID)
		repo.AssertCalled(t, "Update", mock.Anything, staleSensor.ID, mock.Anything)
		repo.AssertExpectations(t)
		assert.NoError(t, err)
	})
}
