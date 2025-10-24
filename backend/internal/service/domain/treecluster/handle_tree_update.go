package treecluster

import (
	"context"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/storage"
)

// HandleCreateTree processes a tree creation event and updates the associated tree cluster if necessary.
//
// If the sensor was previously linked to a different tree with a tree cluster, the previous cluster's watering
// status is recalculated. If the new tree has a tree cluster, an update for that cluster is triggered.
//
// Parameters:
//   - ctx: The request context, enabling logging and tracing.
//   - event: Contains details about the created tree, including its previous and new state.
//
// Returns:
//   - error: An error if updating the previous tree cluster fails; otherwise, nil.
func (s *TreeClusterService) HandleCreateTree(ctx context.Context, event *entities.EventCreateTree) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeClusterService")

	// if the sensor was previously assigned to a different tree, the linked tree cluster must also be updated
	if event.PrevOfSensor != nil && event.PrevOfSensor.TreeCluster != nil {
		if err := s.updateWateringStatusOfPrevTreeCluster(ctx, event.PrevOfSensor.TreeCluster); err != nil {
			return err
		}
	}

	if event.New.TreeCluster == nil {
		return nil
	}

	return s.handleTreeClusterUpdate(ctx, event.New.TreeCluster, event.New)
}

// HandleDeleteTree processes a tree deletion event and updates the affected tree cluster if necessary.
//
// If the deleted tree belonged to a tree cluster, the cluster's watering status is recalculated.
//
// Parameters:
//   - ctx: The request context, enabling logging and tracing.
//   - event: Contains details about the deleted tree, including its previous state.
//
// Returns:
//   - error: An error if updating the tree cluster fails; otherwise, nil.
func (s *TreeClusterService) HandleDeleteTree(ctx context.Context, event *entities.EventDeleteTree) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeClusterService")

	if event.Prev.TreeCluster == nil {
		return nil
	}

	return s.handleTreeClusterUpdate(ctx, event.Prev.TreeCluster, event.Prev)
}

// HandleUpdateTree processes a tree update event and updates the affected tree clusters if necessary.
//
// If the tree's sensor was previously linked to a different tree, the old cluster's watering status is updated.
// If the tree has moved to a different cluster, both the old and new clusters are updated.
//
// Parameters:
//   - ctx: The request context, enabling logging and tracing.
//   - event: Contains details about the tree before and after the update.
//
// Returns:
//   - error: An error if updating any of the affected tree clusters fails; otherwise, nil.
func (s *TreeClusterService) HandleUpdateTree(ctx context.Context, event *entities.EventUpdateTree) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeClusterService")

	// if the sensor was previously assigned to a different tree, the linked tree cluster must also be updated
	if event.PrevOfSensor != nil && event.PrevOfSensor.TreeCluster != nil {
		if err := s.updateWateringStatusOfPrevTreeCluster(ctx, event.PrevOfSensor.TreeCluster); err != nil {
			return err
		}
	}

	if event.Prev.TreeCluster == nil && event.New.TreeCluster == nil {
		return nil
	}

	if s.isNoUpdateNeeded(event) {
		return nil
	}

	if err := s.handleTreeClusterUpdate(ctx, event.Prev.TreeCluster, event.New); err != nil {
		return err
	}

	if event.Prev.TreeCluster != nil && event.New.TreeCluster != nil && event.Prev.TreeCluster.ID != event.New.TreeCluster.ID {
		if err := s.handleTreeClusterUpdate(ctx, event.New.TreeCluster, event.New); err != nil {
			return err
		}
	}

	return nil
}

func (s *TreeClusterService) isNoUpdateNeeded(event *entities.EventUpdateTree) bool {
	treePosSame := event.Prev.Latitude == event.New.Latitude && event.Prev.Longitude == event.New.Longitude
	tcSame := event.Prev.TreeCluster != nil && event.New.TreeCluster != nil && event.Prev.TreeCluster.ID == event.New.TreeCluster.ID
	sensorSame := event.Prev.Sensor == event.New.Sensor
	return treePosSame && tcSame && sensorSame
}

func (s *TreeClusterService) handleTreeClusterUpdate(ctx context.Context, tc *entities.TreeCluster, tree *entities.Tree) error {
	log := logger.GetLogger(ctx)
	if tc == nil || tree.TreeCluster == nil {
		return nil
	}

	wateringStatus, err := s.getWateringStatusOfTreeCluster(ctx, tree.TreeCluster.ID)
	if err != nil {
		log.Error("could not update watering status", "error", err)
	}

	updateFn := func(tc *entities.TreeCluster, repo storage.TreeClusterRepository) (bool, error) {
		if len(tc.Trees) != 0 {
			lat, long, err := repo.GetCenterPoint(ctx, tc.ID)
			if err != nil {
				log.Error("failed to get center point of tree cluster", "error", err, "tree_cluster", tc)
				return false, err
			}

			region, err := s.regionRepo.GetByPoint(ctx, lat, long)
			if err != nil {
				log.Error("can't find region by lat and long", "error", err, "latitude", lat, "longitude", long, "tree_cluster", tc)
				return false, err
			}

			tc.Latitude = &lat
			tc.Longitude = &long
			tc.Region = region
		}
		tc.WateringStatus = wateringStatus
		return true, nil
	}

	if err := s.treeClusterRepo.Update(ctx, tc.ID, updateFn); err == nil {
		log.Info("successfully updated new tree cluster", "cluster_id", tc.ID)
		return s.publishUpdateEvent(ctx, tc)
	}

	return nil
}

func (s *TreeClusterService) updateWateringStatusOfPrevTreeCluster(ctx context.Context, prevTc *entities.TreeCluster) error {
	log := logger.GetLogger(ctx)
	if prevTc == nil {
		return nil
	}

	wateringStatus, err := s.getWateringStatusOfTreeCluster(ctx, prevTc.ID)
	if err != nil {
		log.Error("could not update watering status", "error", err)
	}

	updateFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
		tc.WateringStatus = wateringStatus
		return true, nil
	}

	if err := s.treeClusterRepo.Update(ctx, prevTc.ID, updateFn); err == nil {
		log.Info("successfully updated watering status of previous tree cluster", "cluster_id", prevTc.ID)
		return s.publishUpdateEvent(ctx, prevTc)
	}

	return nil
}
