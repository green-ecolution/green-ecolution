package tree

import (
	"context"
	"testing"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/storage/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestTreeService_HandleNewSensorData(t *testing.T) {
	t.Run("should update watering status on new sensor data event", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTree)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorDataEvent := entities.SensorData{
			SensorID: "sensor-1",
			Data: &entities.MqttPayload{
				Watermarks: []entities.Watermark{
					{Centibar: 30, Depth: 30},
					{Centibar: 40, Depth: 60},
					{Centibar: 50, Depth: 90},
				},
			},
		}

		treeNew := entities.Tree{
			ID:             1,
			PlantingYear:   int32(time.Now().Year() - 2),
			WateringStatus: entities.WateringStatusGood,
		}

		tree := entities.Tree{
			ID:             1,
			PlantingYear:   int32(time.Now().Year() - 2),
			WateringStatus: entities.WateringStatusUnknown,
		}

		event := entities.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, "sensor-1").Return(&tree, nil)
		treeRepo.EXPECT().Update(mock.Anything, mock.Anything, mock.Anything).Return(&treeNew, nil)

		err := svc.HandleNewSensorData(context.Background(), &event)

		assert.NoError(t, err)
		select {
		case receivedEvent := <-ch:
			e, ok := receivedEvent.(entities.EventUpdateTree)
			assert.True(t, ok)
			assert.Equal(t, *e.Prev, tree)
			assert.Equal(t, *e.New, treeNew)
		case <-time.After(100 * time.Millisecond):
			t.Fatal("event was not received")
		}
	})

	t.Run("should not update and not send event if the sensor has no linked tree", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		// event
		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTree)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorDataEvent := entities.SensorData{
			SensorID: "sensor-1",
			Data: &entities.MqttPayload{
				Watermarks: []entities.Watermark{
					{Centibar: 61, Depth: 30},
					{Centibar: 24, Depth: 60},
					{Centibar: 24, Depth: 90},
				},
			},
		}

		event := entities.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, "sensor-1").Return(nil, storage.ErrTreeNotFound)

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
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager)

		// event
		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTree)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorDataEvent := entities.SensorData{
			SensorID: "sensor-1",
			Data: &entities.MqttPayload{
				Watermarks: []entities.Watermark{
					{Centibar: 30, Depth: 30},
					{Centibar: 40, Depth: 60},
					{Centibar: 50, Depth: 90},
				},
			},
		}

		tree := entities.Tree{
			ID:             1,
			PlantingYear:   int32(time.Now().Year() - 2),
			WateringStatus: entities.WateringStatusUnknown,
		}

		event := entities.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, "sensor-1").Return(&tree, nil)
		treeRepo.EXPECT().Update(mock.Anything, mock.Anything, mock.Anything).Return(nil, storage.ErrTreeNotFound)

		// when
		err := svc.HandleNewSensorData(context.Background(), &event)

		// then
		assert.ErrorIs(t, err, storage.ErrTreeNotFound)
		select {
		case <-ch:
			t.Fatal("event was received. It should not have been sent")
		case <-time.After(100 * time.Millisecond):
			assert.True(t, true)
		}
	})
}
