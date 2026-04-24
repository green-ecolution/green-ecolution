package cluster

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	mock "github.com/stretchr/testify/mock"

	clusterDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	treeDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/watering"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

func TestTreeClusterService_HandleUpdateWateringPlan(t *testing.T) {
	t.Run("should update tree cluster last watered and watering status to »just watered«", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		eventManager := worker.NewEventManager(clusterDomain.EventTypeUpdate)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		date := time.Date(2024, 11, 22, 0, 0, 0, 0, time.UTC)
		prevTc := clusterDomain.TreeCluster{
			ID:          1,
			LastWatered: nil,
		}
		prevWp := watering.WateringPlan{
			ID:             1,
			TreeClusterIDs: []int32{prevTc.ID},
			Status:         watering.WateringPlanStatusActive,
			Date:           date,
		}

		updatedWp := watering.WateringPlan{
			ID:             1,
			TreeClusterIDs: []int32{prevTc.ID},
			Status:         watering.WateringPlanStatusFinished,
			Date:           date,
		}

		updatedTc := clusterDomain.TreeCluster{
			ID:          1,
			LastWatered: &date,
		}

		event := watering.NewEventUpdate(&prevWp, &updatedWp)

		// handleTreeClustersUpdate: GetByID for the cluster
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&prevTc, nil).Once()
		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).Return(nil)
		// publishUpdateEvent: GetByID
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&updatedTc, nil).Once()
		// GetByTreeClusterID for updating trees
		treeRepo.EXPECT().GetByTreeClusterID(mock.Anything, int32(1)).Return([]*treeDomain.Tree{}, nil)

		// when
		err := svc.HandleUpdateWateringPlan(context.Background(), &event)

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

	t.Run("should not update tree cluster if status has not changed", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		eventManager := worker.NewEventManager(clusterDomain.EventTypeUpdate)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		date := time.Date(2024, 11, 22, 0, 0, 0, 0, time.UTC)
		prevTc := clusterDomain.TreeCluster{
			ID:          1,
			LastWatered: nil,
		}
		prevWp := watering.WateringPlan{
			ID:             1,
			TreeClusterIDs: []int32{prevTc.ID},
			Status:         watering.WateringPlanStatusActive,
			Date:           date,
		}

		updatedWp := watering.WateringPlan{
			ID:             1,
			TreeClusterIDs: []int32{prevTc.ID},
			Status:         watering.WateringPlanStatusActive,
			Date:           date,
		}

		event := watering.NewEventUpdate(&prevWp, &updatedWp)

		// when
		err := svc.HandleUpdateWateringPlan(context.Background(), &event)

		// then
		assert.NoError(t, err)
		clusterRepo.AssertNotCalled(t, "Update")

		select {
		case <-ch:
			t.Fatalf("event was triggered but shouldn't have been")
		case <-time.After(100 * time.Millisecond):
			assert.True(t, true)
		}
	})

	t.Run("should not update tree cluster if new status is not »finished«", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		eventManager := worker.NewEventManager(clusterDomain.EventTypeUpdate)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		date := time.Date(2024, 11, 22, 0, 0, 0, 0, time.UTC)
		prevTc := clusterDomain.TreeCluster{
			ID:          1,
			LastWatered: nil,
		}
		prevWp := watering.WateringPlan{
			ID:             1,
			TreeClusterIDs: []int32{prevTc.ID},
			Status:         watering.WateringPlanStatusActive,
			Date:           date,
		}

		updatedWp := watering.WateringPlan{
			ID:             1,
			TreeClusterIDs: []int32{prevTc.ID},
			Status:         watering.WateringPlanStatusCanceled,
			Date:           date,
		}

		event := watering.NewEventUpdate(&prevWp, &updatedWp)

		// when
		err := svc.HandleUpdateWateringPlan(context.Background(), &event)

		// then
		assert.NoError(t, err)
		clusterRepo.AssertNotCalled(t, "Update")

		select {
		case <-ch:
			t.Fatalf("event was triggered but shouldn't have been")
		case <-time.After(100 * time.Millisecond):
			assert.True(t, true)
		}
	})

	t.Run("should not update tree cluster if date is not the same", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		eventManager := worker.NewEventManager(clusterDomain.EventTypeUpdate)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(clusterDomain.EventTypeUpdate)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		date := time.Date(2024, 11, 22, 0, 0, 0, 0, time.UTC)
		prevTc := clusterDomain.TreeCluster{
			ID:          1,
			LastWatered: nil,
		}
		prevWp := watering.WateringPlan{
			ID:             1,
			TreeClusterIDs: []int32{prevTc.ID},
			Status:         watering.WateringPlanStatusActive,
			Date:           date,
		}

		updatedWp := watering.WateringPlan{
			ID:             1,
			TreeClusterIDs: []int32{prevTc.ID},
			Status:         watering.WateringPlanStatusCanceled,
			Date:           time.Date(2025, 11, 22, 0, 0, 0, 0, time.UTC),
		}

		event := watering.NewEventUpdate(&prevWp, &updatedWp)

		// when
		err := svc.HandleUpdateWateringPlan(context.Background(), &event)

		// then
		assert.NoError(t, err)
		clusterRepo.AssertNotCalled(t, "Update")

		select {
		case <-ch:
			t.Fatalf("event was triggered but shouldn't have been")
		case <-time.After(100 * time.Millisecond):
			assert.True(t, true)
		}
	})
}
