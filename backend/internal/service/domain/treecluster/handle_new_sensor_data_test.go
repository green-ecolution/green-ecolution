package treecluster

import (
	"context"
	"testing"
	"time"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/storage"
	storageMock "github.com/green-ecolution/backend/internal/storage/_mock"
	"github.com/green-ecolution/backend/internal/worker"
	"github.com/stretchr/testify/assert"
	mock "github.com/stretchr/testify/mock"
)

func TestTreeClusterService_HandleNewSensorData(t *testing.T) {
	t.Run("should update watering status on new sensor data event", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		// event
		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTreeCluster)
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

		tc := &entities.TreeCluster{
			ID: 1,
		}

		tcNew := &entities.TreeCluster{
			ID:             1,
			WateringStatus: entities.WateringStatusGood,
		}

		tree := entities.Tree{
			ID:           1,
			TreeCluster:  tc,
			PlantingYear: int32(time.Now().Year() - 2),
		}

		treeWithSensorID1 := entities.Tree{
			ID: 2,
			TreeCluster: &entities.TreeCluster{
				ID:             1,
				WateringStatus: entities.WateringStatusUnknown,
			},
			Sensor: &entities.Sensor{
				ID: "sensor-1",
			},
			PlantingYear: int32(time.Now().Year() - 2),
		}

		treeWithSensorID2 := entities.Tree{
			ID: 3,
			TreeCluster: &entities.TreeCluster{
				ID:             1,
				WateringStatus: entities.WateringStatusUnknown,
			},
			Sensor: &entities.Sensor{
				ID: "sensor-2",
			},
			PlantingYear: int32(time.Now().Year() - 29), // very old tree
		}

		allLatestSensorData := []*entities.SensorData{
			{
				SensorID: "sensor-1",
				Data: &entities.MqttPayload{
					Watermarks: []entities.Watermark{
						{Centibar: 61, Depth: 30},
						{Centibar: 24, Depth: 60},
						{Centibar: 23, Depth: 90},
					},
				},
			},
			{
				SensorID: "sensor-2",
				Data: &entities.MqttPayload{
					Watermarks: []entities.Watermark{
						{Centibar: 61, Depth: 30},
						{Centibar: 24, Depth: 60},
						{Centibar: 25, Depth: 90},
					},
				},
			},
		}

		event := entities.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, "sensor-1").Return(&tree, nil)
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return(allLatestSensorData, nil)
		treeRepo.EXPECT().GetBySensorIDs(mock.Anything, "sensor-1", "sensor-2").Return([]*entities.Tree{&treeWithSensorID1, &treeWithSensorID2}, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).RunAndReturn(func(ctx context.Context, i int32, f func(*entities.TreeCluster, storage.TreeClusterRepository) (bool, error)) error {
			cluster := entities.TreeCluster{}
			_, err := f(&cluster, clusterRepo)
			assert.NoError(t, err)
			assert.Equal(t, entities.WateringStatusGood, cluster.WateringStatus)
			return nil
		})
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(tcNew, nil)

		// when
		err := svc.HandleNewSensorData(context.Background(), &event)

		// then
		assert.NoError(t, err)
		select {
		case recievedEvent := <-ch:
			e, ok := recievedEvent.(entities.EventUpdateTreeCluster)
			assert.True(t, ok)
			assert.Equal(t, e.Prev, tc)
			assert.Equal(t, e.New, tcNew)
		case <-time.After(100 * time.Millisecond):
			t.Fatal("event was not received")
		}
	})

	t.Run("should update watering status when trees with one latest sensors data in cluster", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		// event
		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTreeCluster)
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

		tc := &entities.TreeCluster{
			ID: 1,
		}

		tcNew := &entities.TreeCluster{
			ID: 1,
		}

		tree := entities.Tree{
			ID:           1,
			TreeCluster:  tc,
			PlantingYear: int32(time.Now().Year() - 2),
		}

		treeWithSensorID1 := entities.Tree{
			ID:             2,
			TreeCluster:    tc,
			WateringStatus: entities.WateringStatusBad,
			Sensor: &entities.Sensor{
				ID: "sensor-1",
			},
			PlantingYear: int32(time.Now().Year() - 1),
		}

		event := entities.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, "sensor-1").Return(&tree, nil)
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return([]*entities.SensorData{&sensorDataEvent}, nil)
		treeRepo.EXPECT().GetBySensorIDs(mock.Anything, "sensor-1").Return([]*entities.Tree{&treeWithSensorID1}, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).RunAndReturn(func(ctx context.Context, i int32, f func(*entities.TreeCluster, storage.TreeClusterRepository) (bool, error)) error {
			cluster := entities.TreeCluster{}
			_, err := f(&cluster, clusterRepo)
			assert.NoError(t, err)
			assert.Equal(t, entities.WateringStatusBad, cluster.WateringStatus)
			return nil
		})
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(tcNew, nil)

		// when
		err := svc.HandleNewSensorData(context.Background(), &event)

		// then
		assert.NoError(t, err)
		select {
		case recievedEvent := <-ch:
			e, ok := recievedEvent.(entities.EventUpdateTreeCluster)
			assert.True(t, ok)
			assert.Equal(t, e.Prev, tc)
			assert.Equal(t, e.New, tcNew)
		case <-time.After(100 * time.Millisecond):
			t.Fatal("event was not received")
		}
	})

	t.Run("should not update and not send event if the watering status did not change", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		// event
		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTreeCluster)
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

		tc := &entities.TreeCluster{
			ID:             1,
			WateringStatus: entities.WateringStatusBad,
		}

		tree := entities.Tree{
			ID:             1,
			TreeCluster:    tc,
			WateringStatus: entities.WateringStatusBad,
			PlantingYear:   int32(time.Now().Year() - 2),
		}

		treeWithSensorID1 := entities.Tree{
			ID:             2,
			TreeCluster:    tc,
			WateringStatus: entities.WateringStatusBad,
			Sensor: &entities.Sensor{
				ID: "sensor-1",
			},
			PlantingYear: int32(time.Now().Year() - 1),
		}

		event := entities.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, "sensor-1").Return(&tree, nil)
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return([]*entities.SensorData{&sensorDataEvent}, nil)
		treeRepo.EXPECT().GetBySensorIDs(mock.Anything, "sensor-1").Return([]*entities.Tree{&treeWithSensorID1}, nil)

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

	t.Run("should not update and not send event if the tree of the sensor has no tree cluster", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		// event
		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTreeCluster)
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

		tree := entities.Tree{
			ID:             1,
			TreeCluster:    nil,
			WateringStatus: entities.WateringStatusBad,
			PlantingYear:   int32(time.Now().Year() - 2),
		}

		event := entities.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, "sensor-1").Return(&tree, nil)

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
}
