package cluster

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	mock "github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

//nolint:gocyclo // function handles multiple test cases and complex event logic, which requires higher complexity to cover all scenarios.
func TestTreeClusterService_HandleUpdateTree(t *testing.T) {
	t.Run("should update tree cluster lat, long, region, watering status and send treecluster update event", func(t *testing.T) {
		clusterRepo, _, regionRepo, eventManager, svc := setupTest(t)

		// event
		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTreeCluster)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		event := entities.NewEventUpdateTree(&prevTree, &updatedTree, nil)
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return(allLatestSensorData, nil)
		clusterRepo.EXPECT().GetCenterPoint(mock.Anything, int32(1)).Return(&updatedTcCoord, nil)
		regionRepo.EXPECT().GetByPoint(mock.Anything, mock.Anything).Return(updatedTc.Region, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).RunAndReturn(func(ctx context.Context, i int32, f func(*entities.TreeCluster, entities.TreeClusterRepository) (bool, error)) error {
			cluster := prevTc
			_, err := f(&cluster, clusterRepo)
			assert.NoError(t, err)
			assert.Equal(t, entities.WateringStatusGood, cluster.WateringStatus)
			return nil
		})
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&updatedTc, nil)

		// when
		err := svc.HandleUpdateTree(context.Background(), &event)

		// then
		assert.NoError(t, err)
		select {
		case recievedEvent, ok := <-ch:
			assert.True(t, ok)
			e := recievedEvent.(entities.EventUpdateTreeCluster)
			assert.Equal(t, e.Prev, &prevTc)
			assert.Equal(t, e.New, &updatedTc)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}
	})

	t.Run("should update tree cluster watering status to unkown and send treecluster update event", func(t *testing.T) {
		clusterRepo, _, _, eventManager, svc := setupTest(t)

		// event
		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTreeCluster)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		event := entities.NewEventUpdateTree(&prevTree, &updatedTree, nil)

		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return(nil, entities.ErrSensorNotFound)
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).RunAndReturn(func(ctx context.Context, i int32, f func(*entities.TreeCluster, entities.TreeClusterRepository) (bool, error)) error {
			cluster := entities.TreeCluster{}
			_, err := f(&cluster, clusterRepo)
			assert.NoError(t, err)
			assert.Equal(t, entities.WateringStatusUnknown, cluster.WateringStatus)
			return nil
		})
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&updatedTc, nil)

		// when
		err := svc.HandleUpdateTree(context.Background(), &event)

		// then
		assert.NoError(t, err)
		select {
		case recievedEvent, ok := <-ch:
			assert.True(t, ok)
			e := recievedEvent.(entities.EventUpdateTreeCluster)
			assert.Equal(t, e.Prev, &prevTc)
			assert.Equal(t, e.New, &updatedTc)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}
	})

	t.Run("should only update watering status to unknown of previous tree cluster linked to sensor", func(t *testing.T) {
		clusterRepo, _, _, eventManager, svc := setupTest(t)

		// event
		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTreeCluster)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		prevTree := entities.Tree{
			TreeCluster: &prevTc,
			Coordinate:  *prevTc.Coordinate,
		}

		updatedTree := entities.Tree{
			TreeCluster: &prevTc,
			Coordinate:  *prevTc.Coordinate,
		}

		event := entities.NewEventUpdateTree(&prevTree, &updatedTree, &prevTreeOfSensor)

		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(2)).Return([]*entities.SensorData{}, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(2), mock.Anything).RunAndReturn(func(ctx context.Context, i int32, f func(*entities.TreeCluster, entities.TreeClusterRepository) (bool, error)) error {
			cluster := entities.TreeCluster{}
			_, err := f(&cluster, clusterRepo)
			assert.NoError(t, err)
			assert.Equal(t, entities.WateringStatusUnknown, cluster.WateringStatus)
			return nil
		})
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(2)).Return(&prevTreeClusterOfSensor, nil)

		// when
		err := svc.HandleUpdateTree(context.Background(), &event)

		// then
		assert.NoError(t, err)
		select {
		case recievedEvent, ok := <-ch:
			assert.True(t, ok)
			e := recievedEvent.(entities.EventUpdateTreeCluster)
			assert.Equal(t, e.Prev, &prevTreeClusterOfSensor)
			assert.Equal(t, e.New, &prevTreeClusterOfSensor)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}
	})

	t.Run("should not update tree cluster if treeclusters in event are nil", func(t *testing.T) {
		clusterRepo, _, regionRepo, eventManager, svc := setupTest(t)

		// event
		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTreeCluster)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		prevWithoutCluster := prevTree
		prevWithoutCluster.TreeCluster = nil

		updatedWithoutCluster := updatedTree
		updatedWithoutCluster.TreeCluster = nil

		event := entities.NewEventUpdateTree(&prevWithoutCluster, &updatedWithoutCluster, nil)

		// when
		err := svc.HandleUpdateTree(context.Background(), &event)

		// then
		assert.NoError(t, err)
		clusterRepo.AssertNotCalled(t, "Update")
		clusterRepo.AssertNotCalled(t, "GetCenterPoint")
		regionRepo.AssertNotCalled(t, "GetByPoint")

		select {
		case <-ch:
			t.Fatalf("event was triggered but shouldn't have been")
		case <-time.After(100 * time.Millisecond):
			assert.True(t, true)
		}
	})

	t.Run("should not update tree cluster if tree has not changed location", func(t *testing.T) {
		clusterRepo, _, regionRepo, eventManager, svc := setupTest(t)

		// event
		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTreeCluster)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		prevTree := entities.Tree{
			TreeCluster: &prevTc,
			Coordinate:  *prevTc.Coordinate,
		}

		updatedTree := entities.Tree{
			TreeCluster: &prevTc,
			Coordinate:  *prevTc.Coordinate,
		}

		event := entities.NewEventUpdateTree(&prevTree, &updatedTree, nil)

		// when
		err := svc.HandleUpdateTree(context.Background(), &event)

		// then
		assert.NoError(t, err)
		clusterRepo.AssertNotCalled(t, "Update")
		clusterRepo.AssertNotCalled(t, "GetCenterPoint")
		regionRepo.AssertNotCalled(t, "GetByPoint")

		select {
		case <-ch:
			t.Fatalf("event was triggered but shouldn't have been")
		case <-time.After(100 * time.Millisecond):
			assert.True(t, true)
		}
	})

	t.Run("should update if tree location is equal but tree has changed treecluster", func(t *testing.T) {
		clusterRepo, _, _, eventManager, svc := setupTest(t)

		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTreeCluster)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		newTcCoord := entities.MustNewCoordinate(54.776366336440255, 9.451084144617182)
		newTc := entities.TreeCluster{
			ID: 2,
			Region: &entities.Region{
				ID:   1,
				Name: "Sandberg",
			},
			Coordinate: &newTcCoord,
			Trees: []*entities.Tree{
				{ID: 1, PlantingYear: entities.MustNewPlantingYear(int32(time.Now().Year() - 2))},
			},
		}

		localUpdatedTree := entities.Tree{
			ID:           1,
			TreeCluster:  &newTc,
			Number:       "T001",
			Coordinate:   entities.MustNewCoordinate(54.811733806341856, 9.482958846410169),
			PlantingYear: entities.MustNewPlantingYear(int32(time.Now().Year() - 2)),
			Sensor:       &entities.Sensor{ID: entities.MustNewSensorID("sensor-1")},
		}

		event := entities.NewEventUpdateTree(&prevTree, &localUpdatedTree, nil)

		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return(allLatestSensorData, nil)
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(2)).Return(allLatestSensorData, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).Return(nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(2), mock.Anything).Return(nil)
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&prevTc, nil)
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(2)).Return(&newTc, nil)

		err := svc.HandleUpdateTree(context.Background(), &event)

		assert.NoError(t, err)
		select {
		case _, ok := <-ch:
			assert.True(t, ok)
		case <-time.After(100 * time.Millisecond):
			t.Fatal("event was not received")
		}
	})

	t.Run("should listen on create new tree event", func(t *testing.T) {
		// given
		eventManager := worker.NewEventManager(entities.EventTypeCreateTree)
		event := entities.NewEventCreateTree(&updatedTree, nil)

		_, ch, _ := eventManager.Subscribe(entities.EventTypeCreateTree)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		// when
		err := eventManager.Publish(context.Background(), event)

		// then
		assert.NoError(t, err)
		select {
		case _, ok := <-ch:
			assert.True(t, ok)
		case <-time.After(100 * time.Millisecond):
			t.Fatal("event was not received")
		}
	})

	t.Run("should listen on update tree event", func(t *testing.T) {
		// given
		eventManager := worker.NewEventManager(entities.EventTypeUpdateTree)
		event := entities.NewEventUpdateTree(&prevTree, &updatedTree, nil)

		_, ch, _ := eventManager.Subscribe(entities.EventTypeUpdateTree)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		// when
		err := eventManager.Publish(context.Background(), event)

		// then
		assert.NoError(t, err)
		select {
		case _, ok := <-ch:
			assert.True(t, ok)
		case <-time.After(100 * time.Millisecond):
			t.Fatal("event was not received")
		}
	})

	t.Run("should listen on delete tree event", func(t *testing.T) {
		// given
		eventManager := worker.NewEventManager(entities.EventTypeDeleteTree)
		event := entities.NewEventDeleteTree(&updatedTree)

		_, ch, _ := eventManager.Subscribe(entities.EventTypeDeleteTree)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		// when
		err := eventManager.Publish(context.Background(), event)

		// then
		assert.NoError(t, err)
		select {
		case _, ok := <-ch:
			assert.True(t, ok)
		case <-time.After(100 * time.Millisecond):
			t.Fatal("event was not received")
		}
	})
}

func setupTest(t *testing.T) (*storageMock.MockTreeClusterRepository, *storageMock.MockTreeRepository, *storageMock.MockRegionRepository, *worker.EventManager, ports.TreeClusterService) {
	clusterRepo := storageMock.NewMockTreeClusterRepository(t)
	treeRepo := storageMock.NewMockTreeRepository(t)
	regionRepo := storageMock.NewMockRegionRepository(t)
	eventManager := worker.NewEventManager(entities.EventTypeUpdateTreeCluster)
	svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)
	return clusterRepo, treeRepo, regionRepo, eventManager, svc
}

var prevTcCoord = entities.MustNewCoordinate(54.776366336440255, 9.451084144617182)
var prevTc = entities.TreeCluster{
	ID: 1,
	Region: &entities.Region{
		ID:   1,
		Name: "Sandberg",
	},
	Coordinate: &prevTcCoord,
	Trees: []*entities.Tree{
		{
			ID:           1,
			PlantingYear: entities.MustNewPlantingYear(int32(time.Now().Year() - 2)),
		},
	},
}

var prevTree = entities.Tree{
	ID:           1,
	TreeCluster:  &prevTc,
	Number:       "T001",
	Coordinate:   entities.MustNewCoordinate(54.776366336440255, 9.451084144617182),
	PlantingYear: entities.MustNewPlantingYear(int32(time.Now().Year() - 2)),
}

var updatedTree = entities.Tree{
	ID:           1,
	TreeCluster:  &prevTc,
	Number:       "T001",
	Coordinate:   entities.MustNewCoordinate(54.811733806341856, 9.482958846410169),
	PlantingYear: entities.MustNewPlantingYear(int32(time.Now().Year() - 2)),
	Sensor: &entities.Sensor{
		ID: entities.MustNewSensorID("sensor-1"),
	},
}

var prevTreeOfSensor = entities.Tree{
	ID:           2,
	TreeCluster:  &prevTreeClusterOfSensor,
	Number:       "T002",
	Coordinate:   entities.MustNewCoordinate(54.811733806341856, 9.482958846410169),
	PlantingYear: entities.MustNewPlantingYear(int32(time.Now().Year() - 2)),
	Sensor: &entities.Sensor{
		ID: entities.MustNewSensorID("sensor-1"),
	},
}

var updatedTcCoord = entities.MustNewCoordinate(54.811733806341856, 9.482958846410169)
var updatedTc = entities.TreeCluster{
	ID: 1,
	Region: &entities.Region{
		ID:   2,
		Name: "Mürwik",
	},
	Coordinate: &updatedTcCoord,
}

var prevTreeClusterOfSensorCoord = entities.MustNewCoordinate(54.811733806341856, 9.482958846410169)
var prevTreeClusterOfSensor = entities.TreeCluster{
	ID: 2,
	Region: &entities.Region{
		ID:   3,
		Name: "Fuerlund",
	},
	Coordinate: &prevTreeClusterOfSensorCoord,
}

var allLatestSensorData = []*entities.SensorData{
	{
		SensorID: entities.MustNewSensorID("sensor-1"),
		Data: &entities.MqttPayload{
			Watermarks: []entities.Watermark{
				{Centibar: 61, Depth: 30},
				{Centibar: 24, Depth: 60},
				{Centibar: 23, Depth: 90},
			},
		},
	},
}
