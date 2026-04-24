package cluster

import (
	"context"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/watering"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

// HandleUpdateWateringPlan processes an update event for a watering plan and updates tree clusters if necessary.
func (s *TreeClusterService) HandleUpdateWateringPlan(ctx context.Context, event *watering.EventUpdate) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeClusterService")

	if event.Prev.Status == event.New.Status ||
		event.Prev.Date != event.New.Date ||
		event.New.Status != watering.WateringPlanStatusFinished ||
		len(event.Prev.TreeClusterIDs) != len(event.New.TreeClusterIDs) {
		return nil
	}

	if err := s.handleTreeClustersUpdate(ctx, event.New.TreeClusterIDs, event.New.Date); err != nil {
		return err
	}

	return nil
}

func (s *TreeClusterService) handleTreeClustersUpdate(ctx context.Context, tcIDs []int32, date time.Time) error {
	log := logger.GetLogger(ctx)
	if len(tcIDs) == 0 {
		return nil
	}

	for _, tcID := range tcIDs {
		tc, err := s.treeClusterRepo.GetByID(ctx, tcID)
		if err != nil {
			log.Error("failed to get tree cluster", "cluster_id", tcID, "err", err)
			continue
		}

		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.WateringStatus = shared.WateringStatusJustWatered
			tc.LastWatered = &date
			return true, nil
		}

		if err := s.treeClusterRepo.Update(ctx, tcID, updateFn); err == nil {
			log.Info("successfully updated last watered date and watering status in tree cluster", "cluster_id", tcID, "last_watered", date)
			err := s.publishUpdateEvent(ctx, tc)
			if err != nil {
				return err
			}
		}

		// Fetch trees belonging to this cluster and update their watering status
		trees, err := s.treeRepo.GetByTreeClusterID(ctx, tcID)
		if err != nil {
			log.Error("failed to get trees for cluster", "cluster_id", tcID, "err", err)
			continue
		}

		for _, tr := range trees {
			_, err := s.treeRepo.Update(ctx, tr.ID, func(t *tree.Tree, _ tree.TreeRepository) (bool, error) {
				log.Debug("updating tree watering status", "prev_status", tr.WateringStatus, "new_status", shared.WateringStatusJustWatered)
				t.WateringStatus = shared.WateringStatusJustWatered
				t.LastWatered = &date
				return true, nil
			})

			if err != nil {
				return err
			}
		}
	}

	return nil
}
