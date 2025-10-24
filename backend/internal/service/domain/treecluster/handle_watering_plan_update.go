package treecluster

import (
	"context"
	"time"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/storage"
)

// HandleUpdateWateringPlan processes an update event for a watering plan and updates tree clusters if necessary.
//
// The function only triggers updates if the watering plan status has changed to "finished," the date remains
// the same, and the linked tree clusters have not changed in number. If these conditions are met, all affected
// tree clusters will have their watering status updated. For each tree cluster in the watering plan as well
// as the trees to this tree cluster, the watering status is set to "just watered," and the last watered date
// is updated. An update event is then published.
//
// Parameters:
//   - ctx: The request context, enabling logging and tracing.
//   - event: Contains details about the updated watering plan, including its previous and new state.
//
// Returns:
//   - error: An error if updating the previous tree cluster fails; otherwise, nil.
func (s *TreeClusterService) HandleUpdateWateringPlan(ctx context.Context, event *entities.EventUpdateWateringPlan) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeClusterService")

	// Tree clusters should only be updated if the status has been changed to ‘finished’
	// and the linked tree clusters and the date have not changed
	if event.Prev.Status == event.New.Status ||
		event.Prev.Date != event.New.Date ||
		event.New.Status != entities.WateringPlanStatusFinished ||
		len(event.Prev.TreeClusters) != len(event.New.TreeClusters) {
		return nil
	}

	if err := s.handleTreeClustersUpdate(ctx, event.New.TreeClusters, event.New.Date); err != nil {
		return err
	}

	return nil
}

func (s *TreeClusterService) handleTreeClustersUpdate(ctx context.Context, tcs []*entities.TreeCluster, date time.Time) error {
	log := logger.GetLogger(ctx)
	if len(tcs) == 0 || tcs == nil {
		return nil
	}

	for _, tc := range tcs {
		updateFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
			tc.WateringStatus = entities.WateringStatusJustWatered
			tc.LastWatered = &date
			return true, nil
		}

		if err := s.treeClusterRepo.Update(ctx, tc.ID, updateFn); err == nil {
			log.Info("successfully updated last watered date and watering status in tree cluster", "cluster_id", tc.ID, "last_watered", date)
			err := s.publishUpdateEvent(ctx, tc)
			if err != nil {
				return err
			}
		}

		for _, tr := range tc.Trees {
			_, err := s.treeRepo.Update(ctx, tr.ID, func(tree *entities.Tree, _ storage.TreeRepository) (bool, error) {
				log.Debug("updating tree watering status", "prev_status", tr.WateringStatus, "new_status", entities.WateringStatusJustWatered)
				tree.WateringStatus = entities.WateringStatusJustWatered
				tree.LastWatered = &date
				return true, nil
			})

			if err != nil {
				return err
			}
		}
	}

	return nil
}
