package sensor_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/service/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/storage/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestSensorService_GetAll(t *testing.T) {
	t.Run("should return all sensors", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		sensorRepo.EXPECT().GetAll(context.Background(), entities.Query{}).Return(TestSensorList, int64(len(TestSensorList)), nil)
		sensors, totalCount, err := svc.GetAll(context.Background(), entities.Query{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, totalCount, int64(len(TestSensorList)))
		assert.Equal(t, TestSensorList, sensors)
	})

	t.Run("should return all sensors by provider", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		sensorRepo.EXPECT().GetAll(context.Background(), entities.Query{Provider: "test-provider"}).Return(TestSensorList, int64(len(TestSensorList)), nil)
		sensors, totalCount, err := svc.GetAll(context.Background(), entities.Query{Provider: "test-provider"})

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestSensorList, sensors)
		assert.Equal(t, totalCount, int64(len(TestSensorList)))
	})

	t.Run("should return error when repository fails", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		sensorRepo.EXPECT().GetAll(context.Background(), entities.Query{}).Return(nil, int64(0), storage.ErrSensorNotFound)
		sensors, totalCount, err := svc.GetAll(context.Background(), entities.Query{})

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
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		sensorRepo.EXPECT().GetAllDataByID(context.Background(), "sensor-1").Return(TestSensorData, nil)
		sensorData, err := svc.GetAllDataByID(context.Background(), "sensor-1")

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestSensorData, sensorData)
	})

	t.Run("should return error when no sensor is found", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		sensorRepo.EXPECT().GetAllDataByID(context.Background(), "sensor-1").Return(nil, storage.ErrSensorNotFound)
		sensorData, err := svc.GetAllDataByID(context.Background(), "sensor-1")

		// then
		assert.Error(t, err)
		assert.Nil(t, sensorData)
		// assert.EqualError(t, err, "500: sensor not found")
	})
}

func TestSensorService_GetByID(t *testing.T) {
	t.Run("should return sensor when found", func(t *testing.T) {
		// given
		id := "sensor-1"
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		sensorRepo.EXPECT().GetByID(context.Background(), id).Return(TestSensor, nil)

		// when
		sensor, err := svc.GetByID(context.Background(), id)

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestSensor, sensor)
	})

	t.Run("should return error if sensor not found", func(t *testing.T) {
		// given
		id := "sensor-1"
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		expectedErr := storage.ErrEntityNotFound("not found")
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
	newSensor := &entities.SensorCreate{
		ID:         "sensor-1",
		Status:     entities.SensorStatusOnline,
		LatestData: TestSensor.LatestData,
		Latitude:   9.446741,
		Longitude:  54.801539,
	}

	t.Run("should successfully create a new sensor", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

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
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		newSensor.LatestData = &entities.SensorData{}

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
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

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
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		newSensor.Status = entities.SensorStatusOffline
		newSensor.ID = ""

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
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		newSensor.ID = "sensor-23"
		newSensor.Status = entities.SensorStatusOffline
		newSensor.Latitude = 200
		newSensor.Longitude = 100

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
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		expectedErr := errors.New("Failed to create sensor")

		newSensor.ID = "sensor-23"
		newSensor.Status = entities.SensorStatusOffline
		newSensor.Latitude = 9.446741
		newSensor.Longitude = 54.801539

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
	updateSensor := &entities.SensorUpdate{
		Status:     entities.SensorStatusOnline,
		LatestData: TestSensor.LatestData,
		Latitude:   9.446741,
		Longitude:  54.801539,
	}

	t.Run("should successfully update a sensor", func(t *testing.T) {
		// given
		id := "sensor-1"
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

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
		id := "notFoundID"
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

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
		id := "sensor-1"
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

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
		id := "sensor-1"
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		updateSensor.Latitude = 200
		updateSensor.Longitude = 200

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
		id := "sensor-1"
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

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
		id := "sensor-1"
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		expectedErr := storage.ErrEntityNotFound("not found")
		sensorRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedErr)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: sensor not found")
	})

	t.Run("should return error if unlinking sensor ID on tree fails", func(t *testing.T) {
		// given
		id := "sensor-1"
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

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
		id := "sensor-1"
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

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
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testSensor := TestSensorNearestTree
		testTree := TestNearestTree

		treeRepo.EXPECT().
			FindNearestTree(context.Background(), mock.Anything, mock.Anything).
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
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

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
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testSensor := TestSensorNearestTree

		treeRepo.EXPECT().
			FindNearestTree(context.Background(), testSensor.Latitude, testSensor.Longitude).
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
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testSensor := TestSensorNearestTree
		testTree := TestNearestTree

		treeRepo.EXPECT().
			FindNearestTree(context.Background(), mock.Anything, mock.Anything).
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
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		ready := svc.Ready()

		// then
		assert.True(t, ready)
	})

	t.Run("should return false if the service is not ready", func(t *testing.T) {
		// give
		svc := sensor.NewSensorService(nil, nil, globalEventManager)

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
		svc := sensor.NewSensorService(repo, treeRepo, globalEventManager)

		staleSensor := &entities.Sensor{
			ID: "sensor-1",
		}
		recentSensor := &entities.Sensor{
			ID: "sensor-2",
		}
		staleSensorData := &entities.SensorData{
			CreatedAt: time.Now().Add(-73 * time.Hour), // Older than 72h
		}
		recentSensorData := &entities.SensorData{
			CreatedAt: time.Now().Add(-1 * time.Hour), // 1 hour ago (not stale)
		}

		expectList := []*entities.Sensor{staleSensor, recentSensor}

		// when
		repo.EXPECT().GetAll(mock.Anything, entities.Query{}).Return(expectList, int64(len(expectList)), nil)
		repo.EXPECT().GetLatestSensorDataBySensorID(mock.Anything, staleSensor.ID).Return(staleSensorData, nil)
		repo.EXPECT().GetLatestSensorDataBySensorID(mock.Anything, recentSensor.ID).Return(recentSensorData, nil)
		repo.EXPECT().Update(mock.Anything, staleSensor.ID, mock.Anything).Return(staleSensor, nil)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.NoError(t, err)
		repo.AssertCalled(t, "GetAll", mock.Anything, entities.Query{})
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
		svc := sensor.NewSensorService(repo, treeRepo, globalEventManager)

		freshSensor := &entities.Sensor{ID: "sensor-1"}
		freshSensorData := &entities.SensorData{
			CreatedAt: time.Now(),
		}

		expectList := []*entities.Sensor{freshSensor}

		// when
		repo.EXPECT().GetAll(mock.Anything, entities.Query{}).Return(expectList, int64(len(expectList)), nil)
		repo.EXPECT().GetLatestSensorDataBySensorID(mock.Anything, freshSensor.ID).Return(freshSensorData, nil)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.NoError(t, err)
		repo.AssertCalled(t, "GetAll", mock.Anything, entities.Query{})
		repo.AssertCalled(t, "GetLatestSensorDataBySensorID", mock.Anything, freshSensor.ID)
		repo.AssertNotCalled(t, "Update")
		repo.AssertExpectations(t)
	})

	t.Run("should return an error when fetching sensors fails", func(t *testing.T) {
		// given
		ctx := context.Background()
		repo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(repo, treeRepo, globalEventManager)

		// when
		expectedErr := errors.New("database error")
		repo.EXPECT().GetAll(mock.Anything, entities.Query{}).Return(nil, int64(0), expectedErr)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.Error(t, err)
		assert.Equal(t, expectedErr, err)
		repo.AssertCalled(t, "GetAll", mock.Anything, entities.Query{})
		repo.AssertNotCalled(t, "GetLatestSensorDataBySensorID")
		repo.AssertNotCalled(t, "Update")
		repo.AssertExpectations(t)
	})

	t.Run("should log an error when fetching latest sensor data fails", func(t *testing.T) {
		// given
		ctx := context.Background()
		repo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(repo, treeRepo, globalEventManager)

		staleSensor := &entities.Sensor{ID: "sensor-1"}
		expectList := []*entities.Sensor{staleSensor}

		expectedErr := errors.New("failed to fetch sensor data")

		// when
		repo.EXPECT().GetAll(mock.Anything, entities.Query{}).Return(expectList, int64(len(expectList)), nil)
		repo.EXPECT().GetLatestSensorDataBySensorID(mock.Anything, staleSensor.ID).Return(nil, expectedErr)

		err := svc.UpdateStatuses(ctx)

		// then
		assert.NoError(t, err)
		repo.AssertCalled(t, "GetAll", mock.Anything, entities.Query{})
		repo.AssertCalled(t, "GetLatestSensorDataBySensorID", mock.Anything, staleSensor.ID)
		repo.AssertNotCalled(t, "Update")
		repo.AssertExpectations(t)
	})

	t.Run("should log an error when updating a sensor fails", func(t *testing.T) {
		// given
		ctx := context.Background()
		repo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(repo, treeRepo, globalEventManager)

		staleSensor := &entities.Sensor{ID: "sensor-1"}
		staleSensorData := &entities.SensorData{
			CreatedAt: time.Now().Add(-100 * time.Hour),
		}

		expectList := []*entities.Sensor{staleSensor}

		// when
		repo.EXPECT().GetAll(mock.Anything, entities.Query{}).Return(expectList, int64(len(expectList)), nil)
		repo.EXPECT().GetLatestSensorDataBySensorID(mock.Anything, staleSensor.ID).Return(staleSensorData, nil)
		repo.EXPECT().Update(mock.Anything, staleSensor.ID, mock.Anything).Return(nil, errors.New("update failed"))

		err := svc.UpdateStatuses(ctx)

		// then
		repo.AssertCalled(t, "GetAll", mock.Anything, entities.Query{})
		repo.AssertCalled(t, "GetLatestSensorDataBySensorID", mock.Anything, staleSensor.ID)
		repo.AssertCalled(t, "Update", mock.Anything, staleSensor.ID, mock.Anything)
		repo.AssertExpectations(t)
		assert.NoError(t, err)
	})
}
