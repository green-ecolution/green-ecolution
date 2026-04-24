package sensor

import (
	"context"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"

	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
)

var globalEventManager = worker.NewEventManager()

func TestNewSensorService(t *testing.T) {
	t.Run("should create a new service", func(t *testing.T) {
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)
		assert.NotNil(t, svc)
	})
}

func TestSensorService_HandleMessage(t *testing.T) {
	t.Run("should process MQTT payload successfully", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testPayLoad := TestListMQTTPayload[0]
		insertData := &sensor.SensorData{
			Data: testPayLoad,
		}

		sensorRepo.EXPECT().GetByID(context.Background(), sensor.MustNewSensorID(testPayLoad.Device)).Return(TestSensor, nil)
		sensorRepo.EXPECT().Update(context.Background(), TestSensor.ID, mock.Anything).Return(TestSensor, nil)
		sensorRepo.EXPECT().InsertSensorData(context.Background(), insertData, TestSensor.ID).Return(nil)
		sensorRepo.EXPECT().GetLatestSensorDataBySensorID(context.Background(), TestSensor.ID).Return(TestSensorData[0], nil)
		treeRepo.EXPECT().FindNearestTrees(mock.Anything, mock.Anything, float64(3), int32(1)).Return([]*tree.TreeWithDistance{{Tree: TestNearestTree, Distance: shared.MustNewDistance(0)}}, nil)
		treeRepo.EXPECT().Update(context.Background(), TestNearestTree.ID, mock.Anything).Return(TestNearestTree, nil)

		// when
		sensorData, err := svc.HandleMessage(context.Background(), testPayLoad)
		sensorStruct, errGetSens := sensorRepo.GetByID(context.Background(), TestSensor.ID)

		// then
		assert.NoError(t, err)
		assert.NoError(t, errGetSens)
		assert.NotNil(t, sensorData)
		assert.NotEmpty(t, sensorData)
		assert.Equal(t, sensorData.Data, insertData.Data)
		assert.NotNil(t, sensorStruct)
		assert.Equal(t, sensorStruct.Coordinate, TestSensor.Coordinate)
		assert.Equal(t, sensorStruct.Status, sensor.SensorStatusOnline)
	})

	t.Run("should return error if sensor update fails", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testPayload := TestListMQTTPayload[0]

		// Use a fresh sensor with different coordinates to ensure updateSensorCoordsAndStatus triggers Update
		testSen := &sensor.Sensor{
			ID:         sensor.MustNewSensorID("sensor001"),
			Coordinate: shared.MustNewCoordinate(54.82124518093376, 9.485702120628517),
			Status:     sensor.SensorStatusOnline,
		}

		sensorRepo.EXPECT().GetByID(context.Background(), sensor.MustNewSensorID(testPayload.Device)).Return(testSen, nil)
		sensorRepo.EXPECT().Update(context.Background(), testSen.ID, mock.Anything).Return(nil, errors.New("update error"))

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
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testPayLoad := TestListMQTTPayload[0]
		insertData := &sensor.SensorData{
			Data: testPayLoad,
		}

		sensorRepo.EXPECT().GetByID(context.Background(), sensor.MustNewSensorID(testPayLoad.Device)).Return(nil, nil).Once()
		sensorRepo.EXPECT().Create(context.Background(), mock.Anything).Return(TestSensor, nil).Once()
		sensorRepo.EXPECT().InsertSensorData(context.Background(), insertData, TestSensor.ID).Return(nil).Once()
		sensorRepo.EXPECT().GetLatestSensorDataBySensorID(context.Background(), TestSensor.ID).Return(TestSensorData[0], nil).Once()
		sensorRepo.EXPECT().GetByID(context.Background(), TestSensor.ID).Return(TestSensor, nil).Once()
		treeRepo.EXPECT().FindNearestTrees(mock.Anything, TestSensor.Coordinate, float64(3), int32(1)).Return([]*tree.TreeWithDistance{{Tree: TestNearestTree, Distance: shared.MustNewDistance(0)}}, nil)
		treeRepo.EXPECT().Update(context.Background(), TestNearestTree.ID, mock.Anything).Return(TestNearestTree, nil)

		// when
		sensorData, err := svc.HandleMessage(context.Background(), testPayLoad)
		sensorStruct, errCreateSens := sensorRepo.GetByID(context.Background(), TestSensor.ID)

		// then
		assert.NoError(t, err)
		assert.NoError(t, errCreateSens)
		assert.NotNil(t, sensorData)
		assert.NotEmpty(t, sensorData)
		assert.Equal(t, sensorData.Data, insertData.Data)
		assert.NotNil(t, sensorStruct)
		assert.Equal(t, sensorStruct.Coordinate, TestSensor.Coordinate)
		assert.Equal(t, sensorStruct.Status, sensor.SensorStatusOnline)
	})

	t.Run("should return error if sensor creation fails", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testPayload := TestListMQTTPayload[0]

		sensorRepo.EXPECT().GetByID(context.Background(), sensor.MustNewSensorID(testPayload.Device)).Return(nil, nil)
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
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

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
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		result, err := svc.HandleMessage(context.Background(), TestMQTTPayLoadInvalidLat)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.Contains(t, err.Error(), ports.ErrValidation.Error())
	})

	t.Run("should return validation error for invalid longitude", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		// when
		result, err := svc.HandleMessage(context.Background(), TestMQTTPayLoadInvalidLong)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.Contains(t, err.Error(), ports.ErrValidation.Error())
	})

	t.Run("should return error if InsertSensorData fails", func(t *testing.T) {
		// given
		sensorRepo := storageMock.NewMockSensorRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		svc := NewSensorService(sensorRepo, treeRepo, globalEventManager)

		testPayLoad := TestListMQTTPayload[0]
		insertData := &sensor.SensorData{
			Data: testPayLoad,
		}

		// Use a fresh sensor with different coordinates to ensure updateSensorCoordsAndStatus triggers Update
		testSen := &sensor.Sensor{
			ID:         sensor.MustNewSensorID("sensor001"),
			Coordinate: shared.MustNewCoordinate(54.82124518093376, 9.485702120628517),
			Status:     sensor.SensorStatusOnline,
		}

		sensorRepo.EXPECT().GetByID(context.Background(), sensor.MustNewSensorID(testPayLoad.Device)).Return(testSen, nil)
		sensorRepo.EXPECT().Update(context.Background(), testSen.ID, mock.Anything).Return(testSen, nil)
		sensorRepo.EXPECT().InsertSensorData(context.Background(), insertData, testSen.ID).Return(errors.New("insert error"))

		// when
		sensorData, err := svc.HandleMessage(context.Background(), testPayLoad)

		// then
		assert.Error(t, err)
		assert.Nil(t, sensorData)
		assert.Contains(t, err.Error(), "insert error")
	})
}
