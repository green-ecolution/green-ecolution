package cluster

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	mock "github.com/stretchr/testify/mock"

	clusterDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	treeDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

func TestTreeClusterService_HandleNewSensorData(t *testing.T) {
	t.Run("should update watering status on new sensor data event", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		sensorRepo := storageMock.NewMockSensorRepository(t)
		eventManager := worker.NewEventManager(clusterDomain.EventTypeUpdate)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, sensorRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorDataEvent := sensor.SensorData{
			SensorID: sensor.MustNewSensorID("sensor-1"),
			Data: &sensor.MqttPayload{
				Watermarks: []sensor.Watermark{
					{Centibar: 30, Depth: 30},
					{Centibar: 40, Depth: 60},
					{Centibar: 50, Depth: 90},
				},
			},
		}

		tree1 := &treeDomain.Tree{
			ID:           2,
			PlantingYear: treeDomain.MustNewPlantingYear(int32(time.Now().Year() - 2)),
			SensorID:     utils.P(sensor.MustNewSensorID("sensor-1")),
		}
		tree2 := &treeDomain.Tree{
			ID:           3,
			PlantingYear: treeDomain.MustNewPlantingYear(int32(time.Now().Year())),
			SensorID:     utils.P(sensor.MustNewSensorID("sensor-2")),
		}

		tc := &clusterDomain.TreeCluster{
			ID:             1,
			WateringStatus: shared.WateringStatusUnknown,
			TreeIDs:        []int32{tree1.ID, tree2.ID},
		}

		tcNew := &clusterDomain.TreeCluster{
			ID:             1,
			WateringStatus: shared.WateringStatusBad,
		}

		tree := &treeDomain.Tree{
			ID:            1,
			TreeClusterID: &tc.ID,
			PlantingYear:  treeDomain.MustNewPlantingYear(int32(time.Now().Year() - 2)),
		}

		allLatestSensorData := []*sensor.SensorData{
			{
				SensorID: sensor.MustNewSensorID("sensor-1"),
				Data: &sensor.MqttPayload{
					Watermarks: []sensor.Watermark{
						{Centibar: 61, Depth: 30},
						{Centibar: 24, Depth: 60},
						{Centibar: 23, Depth: 90},
					},
				},
			},
			{
				SensorID: sensor.MustNewSensorID("sensor-2"),
				Data: &sensor.MqttPayload{
					Watermarks: []sensor.Watermark{
						{Centibar: 61, Depth: 30},
						{Centibar: 24, Depth: 60},
						{Centibar: 25, Depth: 90},
					},
				},
			},
		}

		event := sensor.NewEventNewData(&sensorDataEvent)

		sensorID := sensor.MustNewSensorID("sensor-1")
		treeRepo.EXPECT().GetAll(mock.Anything, treeDomain.TreeQuery{SensorID: &sensorID}).Return([]*treeDomain.Tree{tree}, int64(1), nil)
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(tc, nil).Once()
		clusterID := int32(1)
		treeRepo.EXPECT().GetAll(mock.Anything, treeDomain.TreeQuery{TreeClusterID: &clusterID}).Return([]*treeDomain.Tree{tree1, tree2}, int64(2), nil)
		sensorRepo.EXPECT().GetLatestDataBySensorIDs(mock.Anything, mock.Anything).Return(allLatestSensorData, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).Return(nil)
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(tcNew, nil).Once()

		err := svc.HandleNewSensorData(context.Background(), &event)

		assert.NoError(t, err)
		select {
		case recievedEvent := <-ch:
			e, ok := recievedEvent.(clusterDomain.EventUpdate)
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
		sensorRepo := storageMock.NewMockSensorRepository(t)
		eventManager := worker.NewEventManager(clusterDomain.EventTypeUpdate)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, sensorRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorDataEvent := sensor.SensorData{
			SensorID: sensor.MustNewSensorID("sensor-1"),
			Data: &sensor.MqttPayload{
				Watermarks: []sensor.Watermark{
					{Centibar: 61, Depth: 30},
					{Centibar: 24, Depth: 60},
					{Centibar: 24, Depth: 90},
				},
			},
		}

		treeInCluster := &treeDomain.Tree{
			ID:           2,
			PlantingYear: treeDomain.MustNewPlantingYear(int32(time.Now().Year() - 1)),
			SensorID:     utils.P(sensor.MustNewSensorID("sensor-1")),
		}

		tc := &clusterDomain.TreeCluster{
			ID:             1,
			WateringStatus: shared.WateringStatusUnknown,
			TreeIDs:        []int32{treeInCluster.ID},
		}

		tcNew := &clusterDomain.TreeCluster{ID: 1}

		tree := &treeDomain.Tree{
			ID:            1,
			TreeClusterID: &tc.ID,
			PlantingYear:  treeDomain.MustNewPlantingYear(int32(time.Now().Year() - 2)),
		}

		event := sensor.NewEventNewData(&sensorDataEvent)

		sensorID := sensor.MustNewSensorID("sensor-1")
		treeRepo.EXPECT().GetAll(mock.Anything, treeDomain.TreeQuery{SensorID: &sensorID}).Return([]*treeDomain.Tree{tree}, int64(1), nil)
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(tc, nil).Once()
		clusterID := int32(1)
		treeRepo.EXPECT().GetAll(mock.Anything, treeDomain.TreeQuery{TreeClusterID: &clusterID}).Return([]*treeDomain.Tree{treeInCluster}, int64(1), nil)
		sensorRepo.EXPECT().GetLatestDataBySensorIDs(mock.Anything, []sensor.SensorID{sensor.MustNewSensorID("sensor-1")}).Return([]*sensor.SensorData{&sensorDataEvent}, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).Return(nil)
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(tcNew, nil).Once()

		err := svc.HandleNewSensorData(context.Background(), &event)

		assert.NoError(t, err)
		select {
		case recievedEvent := <-ch:
			e, ok := recievedEvent.(clusterDomain.EventUpdate)
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
		sensorRepo := storageMock.NewMockSensorRepository(t)
		eventManager := worker.NewEventManager(clusterDomain.EventTypeUpdate)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, sensorRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorDataEvent := sensor.SensorData{
			SensorID: sensor.MustNewSensorID("sensor-1"),
			Data: &sensor.MqttPayload{
				Watermarks: []sensor.Watermark{
					{Centibar: 61, Depth: 30},
					{Centibar: 24, Depth: 60},
					{Centibar: 24, Depth: 90},
				},
			},
		}

		treeInCluster := &treeDomain.Tree{
			ID:           2,
			PlantingYear: treeDomain.MustNewPlantingYear(int32(time.Now().Year() - 1)),
			SensorID:     utils.P(sensor.MustNewSensorID("sensor-1")),
		}

		tc := &clusterDomain.TreeCluster{
			ID:             1,
			WateringStatus: shared.WateringStatusBad,
			TreeIDs:        []int32{treeInCluster.ID},
		}

		tree := &treeDomain.Tree{
			ID:            1,
			TreeClusterID: &tc.ID,
			PlantingYear:  treeDomain.MustNewPlantingYear(int32(time.Now().Year() - 2)),
		}

		event := sensor.NewEventNewData(&sensorDataEvent)

		sensorID := sensor.MustNewSensorID("sensor-1")
		treeRepo.EXPECT().GetAll(mock.Anything, treeDomain.TreeQuery{SensorID: &sensorID}).Return([]*treeDomain.Tree{tree}, int64(1), nil)
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(tc, nil)
		clusterID := int32(1)
		treeRepo.EXPECT().GetAll(mock.Anything, treeDomain.TreeQuery{TreeClusterID: &clusterID}).Return([]*treeDomain.Tree{treeInCluster}, int64(1), nil)
		sensorRepo.EXPECT().GetLatestDataBySensorIDs(mock.Anything, []sensor.SensorID{sensor.MustNewSensorID("sensor-1")}).Return([]*sensor.SensorData{&sensorDataEvent}, nil)

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
		sensorRepo := storageMock.NewMockSensorRepository(t)
		eventManager := worker.NewEventManager(clusterDomain.EventTypeUpdate)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, sensorRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		sensorDataEvent := sensor.SensorData{
			SensorID: sensor.MustNewSensorID("sensor-1"),
			Data: &sensor.MqttPayload{
				Watermarks: []sensor.Watermark{
					{Centibar: 61, Depth: 30},
					{Centibar: 24, Depth: 60},
					{Centibar: 24, Depth: 90},
				},
			},
		}

		tree := &treeDomain.Tree{
			ID:            1,
			TreeClusterID: nil,
			PlantingYear:  treeDomain.MustNewPlantingYear(int32(time.Now().Year() - 2)),
		}

		event := sensor.NewEventNewData(&sensorDataEvent)

		sensorID := sensor.MustNewSensorID("sensor-1")
		treeRepo.EXPECT().GetAll(mock.Anything, treeDomain.TreeQuery{SensorID: &sensorID}).Return([]*treeDomain.Tree{tree}, int64(1), nil)

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
