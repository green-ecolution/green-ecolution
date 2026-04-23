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

func TestTreeClusterService_HandleUpdateWateringPlan(t *testing.T) {
	t.Run("should update tree cluster last watered and watering status to »just watered«", func(t *testing.T) {
		clusterRepo := storageMock.NewMockTreeClusterRepository(t)
		treeRepo := storageMock.NewMockTreeRepository(t)
		regionRepo := storageMock.NewMockRegionRepository(t)
		eventManager := worker.NewEventManager(shared.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(shared.EventTypeUpdateTreeCluster)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		date := time.Date(2024, 11, 22, 0, 0, 0, 0, time.UTC)
		prevTc := shared.TreeCluster{
			ID:          1,
			LastWatered: nil,
		}
		prevWp := shared.WateringPlan{
			ID:           1,
			TreeClusters: []*shared.TreeCluster{&prevTc},
			Status:       shared.WateringPlanStatusActive,
			Date:         date,
		}

		updatedWp := shared.WateringPlan{
			ID:           1,
			TreeClusters: []*shared.TreeCluster{&prevTc},
			Status:       shared.WateringPlanStatusFinished,
			Date:         date,
		}

		updatedTc := shared.TreeCluster{
			ID:          1,
			LastWatered: &date,
		}

		event := shared.NewEventUpdateWateringPlan(&prevWp, &updatedWp)

		clusterRepo.EXPECT().Update(mock.Anything, int32(1), mock.Anything).RunAndReturn(func(ctx context.Context, i int32, f func(*shared.TreeCluster, shared.TreeClusterRepository) (bool, error)) error {
			cluster := shared.TreeCluster{}
			_, err := f(&cluster, clusterRepo)
			assert.NoError(t, err)
			assert.Equal(t, shared.WateringStatusJustWatered, cluster.WateringStatus)
			return nil
		})
		clusterRepo.EXPECT().GetByID(mock.Anything, int32(1)).Return(&updatedTc, nil)

		// when
		err := svc.HandleUpdateWateringPlan(context.Background(), &event)

		// then
		assert.NoError(t, err)
		select {
		case recievedEvent, ok := <-ch:
			assert.True(t, ok)
			e := recievedEvent.(shared.EventUpdateTreeCluster)
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
		eventManager := worker.NewEventManager(shared.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(shared.EventTypeUpdateTreeCluster)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		date := time.Date(2024, 11, 22, 0, 0, 0, 0, time.UTC)
		prevTc := shared.TreeCluster{
			ID:          1,
			LastWatered: nil,
		}
		prevWp := shared.WateringPlan{
			ID:           1,
			TreeClusters: []*shared.TreeCluster{&prevTc},
			Status:       shared.WateringPlanStatusActive,
			Date:         date,
		}

		updatedWp := shared.WateringPlan{
			ID:           1,
			TreeClusters: []*shared.TreeCluster{&prevTc},
			Status:       shared.WateringPlanStatusActive,
			Date:         date,
		}

		event := shared.NewEventUpdateWateringPlan(&prevWp, &updatedWp)

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
		eventManager := worker.NewEventManager(shared.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(shared.EventTypeUpdateTreeCluster)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		date := time.Date(2024, 11, 22, 0, 0, 0, 0, time.UTC)
		prevTc := shared.TreeCluster{
			ID:          1,
			LastWatered: nil,
		}
		prevWp := shared.WateringPlan{
			ID:           1,
			TreeClusters: []*shared.TreeCluster{&prevTc},
			Status:       shared.WateringPlanStatusActive,
			Date:         date,
		}

		updatedWp := shared.WateringPlan{
			ID:           1,
			TreeClusters: []*shared.TreeCluster{&prevTc},
			Status:       shared.WateringPlanStatusCanceled,
			Date:         date,
		}

		event := shared.NewEventUpdateWateringPlan(&prevWp, &updatedWp)

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
		eventManager := worker.NewEventManager(shared.EventTypeUpdateTreeCluster)
		svc := NewTreeClusterService(clusterRepo, treeRepo, regionRepo, eventManager)

		_, ch, _ := eventManager.Subscribe(shared.EventTypeUpdateTreeCluster)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go eventManager.Run(ctx)

		date := time.Date(2024, 11, 22, 0, 0, 0, 0, time.UTC)
		prevTc := shared.TreeCluster{
			ID:          1,
			LastWatered: nil,
		}
		prevWp := shared.WateringPlan{
			ID:           1,
			TreeClusters: []*shared.TreeCluster{&prevTc},
			Status:       shared.WateringPlanStatusActive,
			Date:         date,
		}

		updatedWp := shared.WateringPlan{
			ID:           1,
			TreeClusters: []*shared.TreeCluster{&prevTc},
			Status:       shared.WateringPlanStatusCanceled,
			Date:         time.Date(2025, 11, 22, 0, 0, 0, 0, time.UTC),
		}

		event := shared.NewEventUpdateWateringPlan(&prevWp, &updatedWp)

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
