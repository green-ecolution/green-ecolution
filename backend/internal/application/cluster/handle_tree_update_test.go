package cluster

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	mock "github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	clusterDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/region"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	treeDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

//nolint:gocyclo // function handles multiple test cases and complex event logic, which requires higher complexity to cover all scenarios.
func TestTreeClusterService_HandleUpdateTree(t *testing.T) {
	t.Run("should update tree cluster lat, long, region, watering status and send treecluster update event", func(t *testing.T) {
		clusterRepo, treeRepo, regionRepo, eventManager, svc := setupTest(t)

		// event
		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		event := treeDomain.NewEventUpdate(&prevTree, &updatedTree, nil)
		// HandleUpdateTree calls GetByID to fetch prevTc
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&prevTc, nil).Once()
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return(allLatestSensorData, nil)
		treeRepo.EXPECT().GetByTreeClusterID(mock.Anything, int32(1)).Return([]*treeDomain.Tree{&updatedTree}, nil)
		clusterRepo.EXPECT().GetCenterPoint(mock.Anything, int32(1)).Return(&updatedTcCoord, nil)
		regionRepo.EXPECT().GetByPoint(mock.Anything, mock.Anything).Return(&region.Region{ID: 1, Name: "Mürwik"}, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).Return(nil)
		// publishUpdateEvent calls GetByID again to fetch updated tc
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&updatedTc, nil).Once()

		// when
		err := svc.HandleUpdateTree(context.Background(), &event)

		// then
		assert.NoError(t, err)
		select {
		case recievedEvent, ok := <-ch:
			assert.True(t, ok)
			e := recievedEvent.(clusterDomain.EventUpdate)
			assert.Equal(t, e.Prev, &prevTc)
			assert.Equal(t, e.New, &updatedTc)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}
	})

	t.Run("should update tree cluster watering status to unkown and send treecluster update event", func(t *testing.T) {
		clusterRepo, _, regionRepo, eventManager, svc := setupTest(t)

		// event
		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		event := treeDomain.NewEventUpdate(&prevTree, &updatedTree, nil)

		// HandleUpdateTree calls GetByID to fetch prevTc
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&prevTc, nil).Once()
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return(nil, sensor.ErrNotFound)
		clusterRepo.EXPECT().GetCenterPoint(mock.Anything, int32(1)).Return(&updatedTcCoord, nil)
		regionRepo.EXPECT().GetByPoint(mock.Anything, mock.Anything).Return(&region.Region{ID: 1, Name: "Mürwik"}, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).Return(nil)
		// publishUpdateEvent calls GetByID again
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&updatedTc, nil).Once()

		// when
		err := svc.HandleUpdateTree(context.Background(), &event)

		// then
		assert.NoError(t, err)
		select {
		case recievedEvent, ok := <-ch:
			assert.True(t, ok)
			e := recievedEvent.(clusterDomain.EventUpdate)
			assert.Equal(t, e.Prev, &prevTc)
			assert.Equal(t, e.New, &updatedTc)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}
	})

	t.Run("should only update watering status to unknown of previous tree cluster linked to sensor", func(t *testing.T) {
		clusterRepo, treeRepo, _, eventManager, svc := setupTest(t)

		// event
		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		prevTree := treeDomain.Tree{
			TreeClusterID: &prevTc.ID,
			Coordinate:    *prevTc.Coordinate,
		}

		updatedTree := treeDomain.Tree{
			TreeClusterID: &prevTc.ID,
			Coordinate:    *prevTc.Coordinate,
		}

		event := treeDomain.NewEventUpdate(&prevTree, &updatedTree, &prevTreeOfSensor)

		// HandleUpdateTree calls GetByID to fetch prevTreeClusterOfSensor for updateWateringStatusOfPrevTreeCluster
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(2)).Return(&prevTreeClusterOfSensor, nil).Once()
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(2)).Return([]*sensor.SensorData{}, nil)
		treeRepo.EXPECT().GetByTreeClusterID(mock.Anything, int32(2)).Return([]*treeDomain.Tree{}, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(2), mock.Anything).Return(nil)
		// publishUpdateEvent calls GetByID again
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(2)).Return(&prevTreeClusterOfSensor, nil).Once()

		// when
		err := svc.HandleUpdateTree(context.Background(), &event)

		// then
		assert.NoError(t, err)
		select {
		case recievedEvent, ok := <-ch:
			assert.True(t, ok)
			e := recievedEvent.(clusterDomain.EventUpdate)
			assert.Equal(t, e.Prev, &prevTreeClusterOfSensor)
			assert.Equal(t, e.New, &prevTreeClusterOfSensor)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}
	})

	t.Run("should not update tree cluster if treeclusters in event are nil", func(t *testing.T) {
		clusterRepo, _, regionRepo, eventManager, svc := setupTest(t)

		// event
		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		prevWithoutCluster := prevTree
		prevWithoutCluster.TreeClusterID = nil

		updatedWithoutCluster := updatedTree
		updatedWithoutCluster.TreeClusterID = nil

		event := treeDomain.NewEventUpdate(&prevWithoutCluster, &updatedWithoutCluster, nil)

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
		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		prevTree := treeDomain.Tree{
			TreeClusterID: &prevTc.ID,
			Coordinate:    *prevTc.Coordinate,
		}

		updatedTree := treeDomain.Tree{
			TreeClusterID: &prevTc.ID,
			Coordinate:    *prevTc.Coordinate,
		}

		event := treeDomain.NewEventUpdate(&prevTree, &updatedTree, nil)

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
		clusterRepo, treeRepo, regionRepo, eventManager, svc := setupTest(t)

		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		newTcCoord := shared.MustNewCoordinate(54.776366336440255, 9.451084144617182)
		newTc := clusterDomain.TreeCluster{
			ID:         2,
			RegionID:   utils.P(int32(1)),
			Coordinate: &newTcCoord,
			TreeIDs:    []int32{1},
		}

		localUpdatedTree := treeDomain.Tree{
			ID:            1,
			TreeClusterID: &newTc.ID,
			Number:        "T001",
			Coordinate:    shared.MustNewCoordinate(54.811733806341856, 9.482958846410169),
			PlantingYear:  treeDomain.MustNewPlantingYear(int32(time.Now().Year() - 2)),
			SensorID:      utils.P(sensor.MustNewSensorID("sensor-1")),
		}

		event := treeDomain.NewEventUpdate(&prevTree, &localUpdatedTree, nil)

		// HandleUpdateTree: GetByID for prev cluster
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&prevTc, nil).Once()
		// handleTreeClusterUpdate for cluster 1
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(1)).Return(allLatestSensorData, nil)
		treeRepo.EXPECT().GetByTreeClusterID(mock.Anything, int32(1)).Return([]*treeDomain.Tree{&localUpdatedTree}, nil)
		clusterRepo.EXPECT().GetCenterPoint(mock.Anything, int32(1)).Return(&prevTcCoord, nil)
		regionRepo.EXPECT().GetByPoint(mock.Anything, mock.Anything).Return(&region.Region{ID: 1, Name: "Mürwik"}, nil).Maybe()
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).Return(nil)
		// publishUpdateEvent for cluster 1: GetByID
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&prevTc, nil).Once()
		// HandleUpdateTree: GetByID for new cluster
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(2)).Return(&newTc, nil).Once()
		// handleTreeClusterUpdate for cluster 2
		clusterRepo.EXPECT().GetAllLatestSensorDataByClusterID(mock.Anything, int32(2)).Return(allLatestSensorData, nil)
		treeRepo.EXPECT().GetByTreeClusterID(mock.Anything, int32(2)).Return([]*treeDomain.Tree{&localUpdatedTree}, nil)
		clusterRepo.EXPECT().GetCenterPoint(mock.Anything, int32(2)).Return(&newTcCoord, nil)
		clusterRepo.EXPECT().Update(mock.Anything, int32(2), mock.Anything).Return(nil)
		// publishUpdateEvent for cluster 2: GetByID
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(2)).Return(&newTc, nil).Once()

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
		eventManager := worker.NewEventManager(treeDomain.EventTypeCreate)
		event := treeDomain.NewEventCreate(&updatedTree, nil)

		_, ch, _ := eventManager.Subscribe(treeDomain.EventTypeCreate)
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
		eventManager := worker.NewEventManager(treeDomain.EventTypeUpdate)
		event := treeDomain.NewEventUpdate(&prevTree, &updatedTree, nil)

		_, ch, _ := eventManager.Subscribe(treeDomain.EventTypeUpdate)
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
		eventManager := worker.NewEventManager(treeDomain.EventTypeDelete)
		event := treeDomain.NewEventDelete(&updatedTree)

		_, ch, _ := eventManager.Subscribe(treeDomain.EventTypeDelete)
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
	eventManager := worker.NewEventManager(clusterDomain.EventTypeUpdate)
	svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)
	return clusterRepo, treeRepo, regionRepo, eventManager, svc
}

var prevTcCoord = shared.MustNewCoordinate(54.776366336440255, 9.451084144617182)
var prevTc = clusterDomain.TreeCluster{
	ID:         1,
	RegionID:   utils.P(int32(1)),
	Coordinate: &prevTcCoord,
	TreeIDs:    []int32{1},
}

var prevTree = treeDomain.Tree{
	ID:            1,
	TreeClusterID: &prevTc.ID,
	Number:        "T001",
	Coordinate:    shared.MustNewCoordinate(54.776366336440255, 9.451084144617182),
	PlantingYear:  treeDomain.MustNewPlantingYear(int32(time.Now().Year() - 2)),
}

var updatedTree = treeDomain.Tree{
	ID:            1,
	TreeClusterID: &prevTc.ID,
	Number:        "T001",
	Coordinate:    shared.MustNewCoordinate(54.811733806341856, 9.482958846410169),
	PlantingYear:  treeDomain.MustNewPlantingYear(int32(time.Now().Year() - 2)),
	SensorID:      utils.P(sensor.MustNewSensorID("sensor-1")),
}

var prevTreeOfSensor = treeDomain.Tree{
	ID:            2,
	TreeClusterID: &prevTreeClusterOfSensor.ID,
	Number:        "T002",
	Coordinate:    shared.MustNewCoordinate(54.811733806341856, 9.482958846410169),
	PlantingYear:  treeDomain.MustNewPlantingYear(int32(time.Now().Year() - 2)),
	SensorID:      utils.P(sensor.MustNewSensorID("sensor-1")),
}

var updatedTcCoord = shared.MustNewCoordinate(54.811733806341856, 9.482958846410169)
var updatedTc = clusterDomain.TreeCluster{
	ID:         1,
	RegionID:   utils.P(int32(2)),
	Coordinate: &updatedTcCoord,
	TreeIDs:    []int32{1},
}

var prevTreeClusterOfSensorCoord = shared.MustNewCoordinate(54.811733806341856, 9.482958846410169)
var prevTreeClusterOfSensor = clusterDomain.TreeCluster{
	ID:         2,
	RegionID:   utils.P(int32(3)),
	Coordinate: &prevTreeClusterOfSensorCoord,
}

var allLatestSensorData = []*sensor.SensorData{
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
}
