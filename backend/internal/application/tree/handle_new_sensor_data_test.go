package tree

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

var testMapCfg = config.MapConfig{
	NearestTreeMaxRadius:    500,
	NearestTreeDefaultLimit: 10,
	NearestTreeMaxLimit:     50,
}

func TestTreeService_HandleNewSensorData(t *testing.T) {
	t.Run("should update watering status on new sensor data event", func(t *testing.T) {
		treeRepo := storageMock.NewMockTreeRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		eventManager := worker.NewEventManager(shared.EventTypeUpdateTree)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		_, ch, _ := eventManager.Subscribe(shared.EventTypeUpdateTree)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorDataEvent := shared.SensorData{
			SensorID: shared.MustNewSensorID("sensor-1"),
			Data: &shared.MqttPayload{
				Watermarks: []shared.Watermark{
					{Centibar: 30, Depth: 30},
					{Centibar: 40, Depth: 60},
					{Centibar: 50, Depth: 90},
				},
			},
		}

		treeNew := shared.Tree{
			ID:             1,
			PlantingYear:   shared.MustNewPlantingYear(int32(time.Now().Year() - 2)),
			WateringStatus: shared.WateringStatusGood,
		}

		tree := shared.Tree{
			ID:             1,
			PlantingYear:   shared.MustNewPlantingYear(int32(time.Now().Year() - 2)),
			WateringStatus: shared.WateringStatusUnknown,
		}

		event := shared.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, shared.MustNewSensorID("sensor-1")).Return(&tree, nil)
		treeRepo.EXPECT().Update(mock.Anything, mock.Anything, mock.Anything).Return(&treeNew, nil)

		err := svc.HandleNewSensorData(context.Background(), &event)

		assert.NoError(t, err)
		select {
		case receivedEvent := <-ch:
			e, ok := receivedEvent.(shared.EventUpdateTree)
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
		eventManager := worker.NewEventManager(shared.EventTypeUpdateTree)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		// event
		_, ch, _ := eventManager.Subscribe(shared.EventTypeUpdateTree)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorDataEvent := shared.SensorData{
			SensorID: shared.MustNewSensorID("sensor-1"),
			Data: &shared.MqttPayload{
				Watermarks: []shared.Watermark{
					{Centibar: 61, Depth: 30},
					{Centibar: 24, Depth: 60},
					{Centibar: 24, Depth: 90},
				},
			},
		}

		event := shared.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, shared.MustNewSensorID("sensor-1")).Return(nil, shared.ErrTreeNotFound)

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
		eventManager := worker.NewEventManager(shared.EventTypeUpdateTree)
		svc := NewTreeService(treeRepo, sensorRepo, clusterRepo, eventManager, testMapCfg)

		// event
		_, ch, _ := eventManager.Subscribe(shared.EventTypeUpdateTree)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorDataEvent := shared.SensorData{
			SensorID: shared.MustNewSensorID("sensor-1"),
			Data: &shared.MqttPayload{
				Watermarks: []shared.Watermark{
					{Centibar: 30, Depth: 30},
					{Centibar: 40, Depth: 60},
					{Centibar: 50, Depth: 90},
				},
			},
		}

		tree := shared.Tree{
			ID:             1,
			PlantingYear:   shared.MustNewPlantingYear(int32(time.Now().Year() - 2)),
			WateringStatus: shared.WateringStatusUnknown,
		}

		event := shared.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, shared.MustNewSensorID("sensor-1")).Return(&tree, nil)
		treeRepo.EXPECT().Update(mock.Anything, mock.Anything, mock.Anything).Return(nil, shared.ErrTreeNotFound)

		// when
		err := svc.HandleNewSensorData(context.Background(), &event)

		// then
		assert.ErrorIs(t, err, shared.ErrTreeNotFound)
		select {
		case <-ch:
			t.Fatal("event was received. It should not have been sent")
		case <-time.After(100 * time.Millisecond):
			assert.True(t, true)
		}
	})
}
