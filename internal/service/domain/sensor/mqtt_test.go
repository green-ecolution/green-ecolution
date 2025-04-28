package sensor_test

import (
	"context"
	"errors"
	"testing"

	domain "github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/service/domain/sensor"
	"github.com/green-ecolution/backend/internal/worker"
	"github.com/stretchr/testify/mock"

	storageMock "github.com/green-ecolution/backend/internal/storage/_mock"
	"github.com/stretchr/testify/assert"
)

var globalEventManager = worker.NewEventManager()

func TestNewSensorService(t *testing.T) {
	t.Run("should create a new service", func(t *testing.T) {
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)
		assert.NotNil(t, svc)
	})
}

func TestSensorService_HandleMessage(t *testing.T) {
	t.Run("should process MQTT payload successfully", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testPayLoad := TestListMQTTPayload[0]
		insertData := &domain.SensorData{
			Data: testPayLoad,
		}

		sensorRepo.EXPECT().GetByID(context.Background(), testPayLoad.Device).Return(TestSensor, nil)
		sensorRepo.EXPECT().Update(context.Background(), TestSensor.ID, mock.Anything).Return(TestSensor, nil)
		sensorRepo.EXPECT().InsertSensorData(context.Background(), insertData, testPayLoad.Device).Return(nil)
		sensorRepo.EXPECT().GetLatestSensorDataBySensorID(context.Background(), TestSensor.ID).Return(TestSensorData[0], nil)
		treeRepo.EXPECT().FindNearestTree(context.Background(), TestSensor.Latitude, TestSensor.Longitude).Return(TestNearestTree, nil)
		treeRepo.EXPECT().Update(context.Background(), TestNearestTree.ID, mock.Anything).Return(TestNearestTree, nil)

		// when
		sensorData, err := svc.HandleMessage(context.Background(), testPayLoad)
		sensor, errGetSens := sensorRepo.GetByID(context.Background(), TestSensor.ID)

		// then
		assert.NoError(t, err)
		assert.NoError(t, errGetSens)
		assert.NotNil(t, sensorData)
		assert.NotEmpty(t, sensorData)
		assert.Equal(t, sensorData.Data, insertData.Data)
		assert.NotNil(t, sensor)
		assert.Equal(t, sensor.Latitude, TestSensor.Latitude)
		assert.Equal(t, sensor.Longitude, TestSensor.Longitude)
		assert.Equal(t, sensor.Status, domain.SensorStatusOnline)
	})

	t.Run("should return error if sensor update fails", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testPayload := TestListMQTTPayload[0]

		sensorRepo.EXPECT().GetByID(context.Background(), testPayload.Device).Return(TestSensor, nil)
		sensorRepo.EXPECT().Update(context.Background(), TestSensor.ID, mock.Anything).Return(nil, errors.New("update error"))

		// when
		sensorData, err := svc.HandleMessage(context.Background(), testPayload)

		// then
		assert.Error(t, err)
		assert.Nil(t, sensorData)
		assert.Contains(t, err.Error(), "update error")
	})

	t.Run("should process MQTT payload and create a new sensor if not found", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testPayLoad := TestListMQTTPayload[0]
		insertData := &domain.SensorData{
			Data: testPayLoad,
		}

		sensorRepo.EXPECT().GetByID(context.Background(), testPayLoad.Device).Return(nil, nil).Once()
		sensorRepo.EXPECT().Create(context.Background(), mock.Anything).Return(TestSensor, nil).Once()
		sensorRepo.EXPECT().InsertSensorData(context.Background(), insertData, TestSensor.ID).Return(nil).Once()
		sensorRepo.EXPECT().GetLatestSensorDataBySensorID(context.Background(), TestSensor.ID).Return(TestSensorData[0], nil).Once()
		sensorRepo.EXPECT().GetByID(context.Background(), TestSensor.ID).Return(TestSensor, nil).Once()
		treeRepo.EXPECT().FindNearestTree(context.Background(), TestSensor.Latitude, TestSensor.Longitude).Return(TestNearestTree, nil)
		treeRepo.EXPECT().Update(context.Background(), TestNearestTree.ID, mock.Anything).Return(TestNearestTree, nil)

		// when
		sensorData, err := svc.HandleMessage(context.Background(), testPayLoad)
		sensor, errCreateSens := sensorRepo.GetByID(context.Background(), TestSensor.ID)

		// then
		assert.NoError(t, err)
		assert.NoError(t, errCreateSens)
		assert.NotNil(t, sensorData)
		assert.NotEmpty(t, sensorData)
		assert.Equal(t, sensorData.Data, insertData.Data)
		assert.NotNil(t, sensor)
		assert.Equal(t, sensor.Latitude, TestSensor.Latitude)
		assert.Equal(t, sensor.Longitude, TestSensor.Longitude)
		assert.Equal(t, sensor.Status, domain.SensorStatusOnline)
	})

	t.Run("should return error if sensor creation fails", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testPayload := TestListMQTTPayload[0]

		sensorRepo.EXPECT().GetByID(context.Background(), testPayload.Device).Return(nil, nil)
		sensorRepo.EXPECT().Create(context.Background(), mock.Anything).Return(nil, errors.New("create error"))

		// when
		sensorData, err := svc.HandleMessage(context.Background(), testPayload)

		// then
		assert.Error(t, err)
		assert.Nil(t, sensorData)
		assert.Contains(t, err.Error(), "create error")
	})

	t.Run("should return error when payload is nil", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		result, err := svc.HandleMessage(context.Background(), nil)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return validation error for invalid latitude", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		result, err := svc.HandleMessage(context.Background(), TestMQTTPayLoadInvalidLat)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.Contains(t, err.Error(), service.ErrValidation.Error())
	})

	t.Run("should return validation error for invalid longitude", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		result, err := svc.HandleMessage(context.Background(), TestMQTTPayLoadInvalidLong)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.Contains(t, err.Error(), service.ErrValidation.Error())
	})

	t.Run("should return error if InsertSensorData fails", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := sensor.NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testPayLoad := TestListMQTTPayload[0]
		insertData := &domain.SensorData{
			Data: testPayLoad,
		}

		sensorRepo.EXPECT().GetByID(context.Background(), testPayLoad.Device).Return(TestSensor, nil)
		sensorRepo.EXPECT().Update(context.Background(), TestSensor.ID, mock.Anything).Return(TestSensor, nil)
		sensorRepo.EXPECT().InsertSensorData(context.Background(), insertData, testPayLoad.Device).Return(errors.New("insert error"))

		// when
		sensorData, err := svc.HandleMessage(context.Background(), testPayLoad)

		// then
		assert.Error(t, err)
		assert.Nil(t, sensorData)
		assert.Contains(t, err.Error(), "insert error")
	})
}
