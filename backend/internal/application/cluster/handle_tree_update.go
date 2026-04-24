package cluster

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

// HandleCreateTree processes a tree creation event and updates the associated tree cluster if necessary.
func (s *TreeClusterService) HandleCreateTree(ctx context.Context, event *tree.EventCreate) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeClusterService")

	if err := s.handleSensorPrevClusterUpdate(ctx, event.PrevOfSensor); err != nil {
		return err
	}

	if event.New.TreeClusterID == nil {
		return nil
	}

	tc, err := s.treeClusterRepo.GetByID(ctx, *event.New.TreeClusterID)
	if err != nil {
		log.Error("failed to get tree cluster for new tree", "cluster_id", *event.New.TreeClusterID, "err", err)
		return err
	}

	return s.handleTreeClusterUpdate(ctx, tc, event.New)
}

// HandleDeleteTree processes a tree deletion event and updates the affected tree cluster if necessary.
func (s *TreeClusterService) HandleDeleteTree(ctx context.Context, event *tree.EventDelete) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeClusterService")

	if event.Prev.TreeClusterID == nil {
		return nil
	}

	tc, err := s.treeClusterRepo.GetByID(ctx, *event.Prev.TreeClusterID)
	if err != nil {
		log.Error("failed to get tree cluster for deleted tree", "cluster_id", *event.Prev.TreeClusterID, "err", err)
		return err
	}

	return s.handleTreeClusterUpdate(ctx, tc, event.Prev)
}

// HandleUpdateTree processes a tree update event and updates the affected tree clusters if necessary.
func (s *TreeClusterService) HandleUpdateTree(ctx context.Context, event *tree.EventUpdate) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeClusterService")

	if err := s.handleSensorPrevClusterUpdate(ctx, event.PrevOfSensor); err != nil {
		return err
	}

	if event.Prev.TreeClusterID == nil && event.New.TreeClusterID == nil {
		return nil
	}

	if event.Prev.TreeClusterID != nil && !needsPositionUpdate(event.Prev, event.New) {
		return nil
	}

	if event.Prev.TreeClusterID != nil {
		prevTc, err := s.treeClusterRepo.GetByID(ctx, *event.Prev.TreeClusterID)
		if err != nil {
			log.Error("failed to get prev tree cluster", "cluster_id", *event.Prev.TreeClusterID, "err", err)
			return err
		}
		if err := s.handleTreeClusterUpdate(ctx, prevTc, event.New); err != nil {
			return err
		}
	}

	// If tree moved to a different cluster, update the new one too
	if event.Prev.TreeClusterID != nil && event.New.TreeClusterID != nil && *event.Prev.TreeClusterID != *event.New.TreeClusterID {
		newTc, err := s.treeClusterRepo.GetByID(ctx, *event.New.TreeClusterID)
		if err != nil {
			log.Error("failed to get new tree cluster", "cluster_id", *event.New.TreeClusterID, "err", err)
			return err
		}
		if err := s.handleTreeClusterUpdate(ctx, newTc, event.New); err != nil {
			return err
		}
	}

	return nil
}

func (s *TreeClusterService) handleSensorPrevClusterUpdate(ctx context.Context, prevOfSensor *tree.Tree) error {
	if prevOfSensor == nil || prevOfSensor.TreeClusterID == nil {
		return nil
	}

	prevTc, err := s.treeClusterRepo.GetByID(ctx, *prevOfSensor.TreeClusterID)
	if err != nil {
		return nil
	}

	return s.updateWateringStatusOfPrevTreeCluster(ctx, prevTc)
}

func (s *TreeClusterService) handleTreeClusterUpdate(ctx context.Context, tc *cluster.TreeCluster, t *tree.Tree) error {
	log := logger.GetLogger(ctx)
	if tc == nil || t.TreeClusterID == nil {
		return nil
	}

	wateringStatus, err := s.getWateringStatusOfTreeCluster(ctx, tc.ID)
	if err != nil {
		log.Error("could not calculate watering status", "error", err)
	}

	if len(tc.TreeIDs) != 0 {
		coord, err := s.treeClusterRepo.GetCenterPoint(ctx, tc.ID)
		if err != nil {
			log.Error("failed to get center point of tree cluster", "error", err, "tree_cluster", tc)
			return err
		}

		region, err := s.regionRepo.GetByPoint(ctx, *coord)
		if err != nil {
			log.Error("can't find region by coordinate", "error", err, "coordinate", coord, "tree_cluster", tc)
			return err
		}

		tc.Coordinate = coord
		if region != nil {
			tc.RegionID = &region.ID
		}
	}
	tc.WateringStatus = wateringStatus

	if err := s.treeClusterRepo.Update(ctx, tc.ID, tc); err == nil {
		log.Info("successfully updated tree cluster", "cluster_id", tc.ID)
		return s.publishUpdateEvent(ctx, tc)
	}

	return nil
}

func (s *TreeClusterService) updateWateringStatusOfPrevTreeCluster(ctx context.Context, prevTc *cluster.TreeCluster) error {
	log := logger.GetLogger(ctx)
	if prevTc == nil {
		return nil
	}

	wateringStatus, err := s.getWateringStatusOfTreeCluster(ctx, prevTc.ID)
	if err != nil {
		log.Error("could not update watering status", "error", err)
	}

	prevTc.WateringStatus = wateringStatus
	if err := s.treeClusterRepo.Update(ctx, prevTc.ID, prevTc); err == nil {
		log.Info("successfully updated watering status of previous tree cluster", "cluster_id", prevTc.ID)
		return s.publishUpdateEvent(ctx, prevTc)
	}

	return nil
}

// needsPositionUpdate checks whether a tree change warrants recalculating the cluster position.
// Compares tree cluster IDs and sensor IDs between old and new state.
func needsPositionUpdate(prev, newTree *tree.Tree) bool {
	// Cluster assignment changed
	if !int32PtrEqual(prev.TreeClusterID, newTree.TreeClusterID) {
		return true
	}

	// Sensor assignment changed
	if (prev.SensorID == nil) != (newTree.SensorID == nil) {
		return true
	}
	if prev.SensorID != nil && newTree.SensorID != nil && *prev.SensorID != *newTree.SensorID {
		return true
	}

	// Coordinate changed
	if prev.Coordinate != newTree.Coordinate {
		return true
	}

	return false
}

func int32PtrEqual(a, b *int32) bool {
	if a == nil && b == nil {
		return true
	}
	if a == nil || b == nil {
		return false
	}
	return *a == *b
}
