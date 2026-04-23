package cluster

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	mock "github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

func TestTreeClusterService_HandleNewSensorData(t *testing.T) {
	t.Run("should update watering status on new sensor data event", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		eventManager := worker.NewEventManager(shared.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(shared.EventTypeUpdateTreeCluster)
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

		tree1 := &shared.Tree{
			ID:           2,
			PlantingYear: shared.MustNewPlantingYear(int32(time.Now().Year() - 2)),
		}
		tree2 := &shared.Tree{
			ID:           3,
			PlantingYear: shared.MustNewPlantingYear(int32(time.Now().Year())),
		}

		tc := &shared.TreeCluster{
			ID:             1,
			WateringStatus: shared.WateringStatusUnknown,
			Trees:          []*shared.Tree{tree1, tree2},
		}

		tcNew := &shared.TreeCluster{
			ID:             1,
			WateringStatus: shared.WateringStatusGood,
		}

		tree := &shared.Tree{
			ID:           1,
			TreeCluster:  tc,
			PlantingYear: shared.MustNewPlantingYear(int32(time.Now().Year() - 2)),
		}

		allLatestSensorData := []*shared.SensorData{
			{
				SensorID: shared.MustNewSensorID("sensor-1"),
				Data: &shared.MqttPayload{
					Watermarks: []shared.Watermark{
						{Centibar: 61, Depth: 30},
						{Centibar: 24, Depth: 60},
						{Centibar: 23, Depth: 90},
					},
				},
			},
			{
				SensorID: shared.MustNewSensorID("sensor-2"),
				Data: &shared.MqttPayload{
					Watermarks: []shared.Watermark{
						{Centibar: 61, Depth: 30},
						{Centibar: 24, Depth: 60},
						{Centibar: 25, Depth: 90},
					},
				},
			},
		}

		event := shared.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, shared.MustNewSensorID("sensor-1")).Return(tree, nil)
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return(allLatestSensorData, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).RunAndReturn(func(ctx context.Context, i int32, f func(*shared.TreeCluster, shared.TreeClusterRepository) (bool, error)) error {
			cluster := *tc
			_, err := f(&cluster, clusterRepo)
			assert.NoError(t, err)
			assert.Equal(t, shared.WateringStatusGood, cluster.WateringStatus)
			return nil
		})
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(tcNew, nil)

		err := svc.HandleNewSensorData(context.Background(), &event)

		assert.NoError(t, err)
		select {
		case recievedEvent := <-ch:
			e, ok := recievedEvent.(shared.EventUpdateTreeCluster)
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
		eventManager := worker.NewEventManager(shared.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(shared.EventTypeUpdateTreeCluster)
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

		treeInCluster := &shared.Tree{
			ID:           2,
			PlantingYear: shared.MustNewPlantingYear(int32(time.Now().Year() - 1)),
		}

		tc := &shared.TreeCluster{
			ID:             1,
			WateringStatus: shared.WateringStatusUnknown,
			Trees:          []*shared.Tree{treeInCluster},
		}

		tcNew := &shared.TreeCluster{ID: 1}

		tree := &shared.Tree{
			ID:           1,
			TreeCluster:  tc,
			PlantingYear: shared.MustNewPlantingYear(int32(time.Now().Year() - 2)),
		}

		event := shared.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, shared.MustNewSensorID("sensor-1")).Return(tree, nil)
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return([]*shared.SensorData{&sensorDataEvent}, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).RunAndReturn(func(ctx context.Context, i int32, f func(*shared.TreeCluster, shared.TreeClusterRepository) (bool, error)) error {
			cluster := *tc
			_, err := f(&cluster, clusterRepo)
			assert.NoError(t, err)
			assert.Equal(t, shared.WateringStatusBad, cluster.WateringStatus)
			return nil
		})
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(tcNew, nil)

		err := svc.HandleNewSensorData(context.Background(), &event)

		assert.NoError(t, err)
		select {
		case recievedEvent := <-ch:
			e, ok := recievedEvent.(shared.EventUpdateTreeCluster)
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
		eventManager := worker.NewEventManager(shared.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(shared.EventTypeUpdateTreeCluster)
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

		treeInCluster := &shared.Tree{
			ID:           2,
			PlantingYear: shared.MustNewPlantingYear(int32(time.Now().Year() - 1)),
		}

		tc := &shared.TreeCluster{
			ID:             1,
			WateringStatus: shared.WateringStatusBad,
			Trees:          []*shared.Tree{treeInCluster},
		}

		tree := &shared.Tree{
			ID:           1,
			TreeCluster:  tc,
			PlantingYear: shared.MustNewPlantingYear(int32(time.Now().Year() - 2)),
		}

		event := shared.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, shared.MustNewSensorID("sensor-1")).Return(tree, nil)
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return([]*shared.SensorData{&sensorDataEvent}, nil)

		err := svc.HandleNewSensorData(context.Background(), &event)

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
		eventManager := worker.NewEventManager(shared.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(shared.EventTypeUpdateTreeCluster)
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

		tree := &shared.Tree{
			ID:           1,
			TreeCluster:  nil,
			PlantingYear: shared.MustNewPlantingYear(int32(time.Now().Year() - 2)),
		}

		event := shared.NewEventSensorData(&sensorDataEvent)

		treeRepo.EXPECT().GetBySensorID(mock.Anything, shared.MustNewSensorID("sensor-1")).Return(tree, nil)

		err := svc.HandleNewSensorData(context.Background(), &event)

		assert.NoError(t, err)
		select {
		case <-ch:
			t.Fatal("event was received. It should not have been sent")
		case <-time.After(100 * time.Millisecond):
			assert.True(t, true)
		}
	})
}
