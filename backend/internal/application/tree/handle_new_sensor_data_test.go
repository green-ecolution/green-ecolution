package tree

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

func TestTreeService_HandleNewSensorData(t *testing.T) {
	t.Run("should update watering status on new sensor data event", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		_, ch, _ := eventManager.Subscribe(tree.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorID := sensor.MustNewSensorID("sensor-1")
		sensorDataEvent := sensor.SensorData{
			SensorID: sensorID,
			Data: &sensor.MqttPayload{
				Watermarks: []sensor.Watermark{
					{Centibar: 30, Depth: 30},
					{Centibar: 40, Depth: 60},
					{Centibar: 50, Depth: 90},
				},
			},
		}

		treeNew := tree.Tree{
			ID:             1,
			PlantingYear:   tree.MustNewPlantingYear(int32(time.Now().Year() - 2)),
			WateringStatus: shared.WateringStatusGood,
		}

		treeStruct := tree.Tree{
			ID:             1,
			PlantingYear:   tree.MustNewPlantingYear(int32(time.Now().Year() - 2)),
			WateringStatus: shared.WateringStatusUnknown,
		}

		event := sensor.NewEventNewData(&sensorDataEvent)

		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{SensorID: &sensorID}).Return([]*tree.Tree{&treeStruct}, int64(1), nil)
		treeRepo.EXPECT().Update(mock.Anything, mock.Anything, mock.Anything).Return(&treeNew, nil)

		err := svc.HandleNewSensorData(context.Background(), &event)

		assert.NoError(t, err)
		select {
		case receivedEvent := <-ch:
			e, ok := receivedEvent.(tree.EventUpdate)
			assert.True(t, ok)
			assert.Equal(t, *e.Prev, treeStruct)
			assert.Equal(t, *e.New, treeNew)
		case <-time.After(100 * time.Millisecond):
			t.Fatal("event was not received")
		}
	})

	t.Run("should not update and not send event if the sensor has no linked tree", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		// event
		_, ch, _ := eventManager.Subscribe(tree.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorID := sensor.MustNewSensorID("sensor-1")
		sensorDataEvent := sensor.SensorData{
			SensorID: sensorID,
			Data: &sensor.MqttPayload{
				Watermarks: []sensor.Watermark{
					{Centibar: 61, Depth: 30},
					{Centibar: 24, Depth: 60},
					{Centibar: 24, Depth: 90},
				},
			},
		}

		event := sensor.NewEventNewData(&sensorDataEvent)

		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{SensorID: &sensorID}).Return([]*tree.Tree{}, int64(0), nil)

		// when
		err := svc.HandleNewSensorData(context.Background(), &event)

		// then
		assert.NoError(t, err)
		select {
		case <-ch:
			t.Fatal("event was received. It should not have been sent")
		case <-time.After(100 * time.Millisecond):
			assert.True(t, true)
		}
	})

	t.Run("should not update and not send event if tree could not be updated", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(tree.EventTypeUpdate)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		// event
		_, ch, _ := eventManager.Subscribe(tree.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorID := sensor.MustNewSensorID("sensor-1")
		sensorDataEvent := sensor.SensorData{
			SensorID: sensorID,
			Data: &sensor.MqttPayload{
				Watermarks: []sensor.Watermark{
					{Centibar: 30, Depth: 30},
					{Centibar: 40, Depth: 60},
					{Centibar: 50, Depth: 90},
				},
			},
		}

		treeStruct := tree.Tree{
			ID:             1,
			PlantingYear:   tree.MustNewPlantingYear(int32(time.Now().Year() - 2)),
			WateringStatus: shared.WateringStatusUnknown,
		}

		event := sensor.NewEventNewData(&sensorDataEvent)

		treeRepo.EXPECT().GetAll(mock.Anything, tree.TreeQuery{SensorID: &sensorID}).Return([]*tree.Tree{&treeStruct}, int64(1), nil)
		treeRepo.EXPECT().Update(mock.Anything, mock.Anything, mock.Anything).Return(nil, tree.ErrNotFound)

		// when
		err := svc.HandleNewSensorData(context.Background(), &event)

		// then
		assert.ErrorIs(t, err, tree.ErrNotFound)
		select {
		case <-ch:
			t.Fatal("event was received. It should not have been sent")
		case <-time.After(100 * time.Millisecond):
			assert.True(t, true)
		}
	})
}
