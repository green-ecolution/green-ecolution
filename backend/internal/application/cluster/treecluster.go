package cluster

import (
	"context"
	"log/slog"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/region"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

type TreeClusterService struct {
	treeClusterRepo cluster.TreeClusterRepository
	treeRepo        tree.TreeRepository
	regionRepo      region.RegionRepository
	eventManager    *worker.EventManager
}

func NewTreeClusterService(
	treeClusterRepo cluster.TreeClusterRepository,
	treeRepo tree.TreeRepository,
	regionRepo region.RegionRepository,
	eventManager *worker.EventManager,
) ports.TreeClusterService {
	return &TreeClusterService{
		treeClusterRepo: treeClusterRepo,
		treeRepo:        treeRepo,
		regionRepo:      regionRepo,
		eventManager:    eventManager,
	}
}

func (s *TreeClusterService) GetAll(ctx context.Context, filter cluster.TreeClusterQuery) ([]*cluster.TreeCluster, int64, error) {
	log := logger.GetLogger(ctx)

	treeClusters, totalCount, err := s.treeClusterRepo.GetAll(ctx, filter)
	if err != nil {
		log.Debug("failed to fetch tree clsuters", "error", err)
		return nil, 0, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}
	return treeClusters, totalCount, nil
}

func (s *TreeClusterService) GetByID(ctx context.Context, id int32) (*cluster.TreeCluster, error) {
	log := logger.GetLogger(ctx)
	treeCluster, err := s.treeClusterRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to fetch tree cluster by id", "error", err, "cluster_id", id)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return treeCluster, nil
}

func (s *TreeClusterService) publishUpdateEvent(ctx context.Context, prevTc *cluster.TreeCluster) error {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", cluster.EventTypeUpdate, "service", "TreeClusterService")
	updatedTc, err := s.GetByID(ctx, prevTc.ID)
	if err != nil {
		return err
	}

	event := cluster.NewEventUpdate(prevTc, updatedTc)
	err = s.eventManager.Publish(ctx, event)
	if err != nil {
		log.Error("error while sending event after updating tree cluster", "err", err)
	}

	return nil
}

func (s *TreeClusterService) Create(ctx context.Context, createTc *cluster.TreeClusterCreate) (*cluster.TreeCluster, error) {
	log := logger.GetLogger(ctx)

	trees, err := s.getTrees(ctx, createTc.TreeIDs)
	if err != nil {
		log.Debug("failed to get trees inside the tree cluster", "error", err, "tree_ids", createTc.TreeIDs)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	treeIDs := treeIDsFromTrees(trees)

	c, err := s.treeClusterRepo.Create(ctx, func(tc *cluster.TreeCluster, repo cluster.TreeClusterRepository) (bool, error) {
		if err = s.handlePrevTreeLocation(ctx, trees, repo.Update); err != nil {
			log.Debug("failed to update prev tree location", "error", err, "trees", trees, "tree_cluster", tc)
			return false, ports.MapError(ctx, err, ports.ErrorLogAll)
		}

		tc.TreeIDs = treeIDs
		tc.Name = createTc.Name
		tc.Address = createTc.Address
		tc.Description = createTc.Description
		tc.SoilCondition = createTc.SoilCondition
		tc.Provider = createTc.Provider
		tc.AdditionalInfo = createTc.AdditionalInfo

		log.Debug("creating tree cluster with following attributes",
			"tree_ids", createTc.TreeIDs,
			"name", createTc.Name,
			"address", createTc.Address,
			"description", createTc.Description,
			"soil_condition", createTc.SoilCondition,
		)

		return true, nil
	})

	if err != nil {
		log.Debug("failed to create tree cluster", "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	if err := s.UpdateWateringStatuses(ctx); err != nil {
		log.Warn("failed to update watering status after creating tree cluster", "error", err, "cluster_id", c.ID)
	}

	if err := s.updateTreeClusterPosition(ctx, c.ID); err != nil {
		log.Debug("error while update the cluster locations", "error", err, "cluster_id", c.ID)
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	log.Info("tree cluster created successfully", "cluster_id", c.ID)
	return c, nil
}

func (s *TreeClusterService) Update(ctx context.Context, id int32, tcUpdate *cluster.TreeClusterUpdate) (*cluster.TreeCluster, error) {
	log := logger.GetLogger(ctx)

	trees, err := s.getTrees(ctx, tcUpdate.TreeIDs)
	if err != nil {
		log.Debug("failed to get trees inside the tree cluster", "error", err, "tree_ids", tcUpdate.TreeIDs)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	treeIDs := treeIDsFromTrees(trees)

	prevTc, err := s.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to get exiting tree cluster", "error", err, "cluster_id", id)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	err = s.treeClusterRepo.Update(ctx, id, func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
		tc.TreeIDs = treeIDs
		tc.Name = tcUpdate.Name
		tc.Address = tcUpdate.Address
		tc.Description = tcUpdate.Description
		tc.SoilCondition = tcUpdate.SoilCondition
		tc.Provider = tcUpdate.Provider
		tc.AdditionalInfo = tcUpdate.AdditionalInfo

		log.Debug("updating tree cluster with following attributes",
			"cluster_id", id,
			"name", tcUpdate.Name,
			"address", tcUpdate.Address,
			"description", tcUpdate.Description,
			"soil_condition", tcUpdate.SoilCondition,
			"provider", tcUpdate.Provider,
			"additional_info", tcUpdate.AdditionalInfo,
		)

		return true, nil
	})

	if err != nil {
		log.Debug("failed to update tree cluster", "error", err, "cluster_id", id)
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}
	log.Info("tree cluster updated successfully", "cluster_id", id)

	if err := s.UpdateWateringStatuses(ctx); err != nil {
		log.Warn("failed to update watering status after updating tree cluster", "error", err, "cluster_id", id)
	}

	// Collect affected clusters: prev clusters of each tree + prevTc
	var eventTreeClusterIDs []int32
	if len(trees) > 0 {
		for _, t := range trees {
			if t.TreeClusterID != nil && *t.TreeClusterID != id {
				eventTreeClusterIDs = append(eventTreeClusterIDs, *t.TreeClusterID)
			}
		}
	}

	// Deduplicate and process each affected cluster
	visitedIDs := make(map[int32]bool)
	for _, tcID := range eventTreeClusterIDs {
		if visitedIDs[tcID] {
			continue
		}
		visitedIDs[tcID] = true

		eTC, err := s.treeClusterRepo.GetByID(ctx, tcID)
		if err != nil {
			log.Error("failed to get affected tree cluster", "cluster_id", tcID, "err", err)
			continue
		}

		if err = s.updateTreeClusterPosition(ctx, eTC.ID); err != nil {
			log.Error("error while update the cluster locations", "error", err, "cluster_id", eTC.ID)
			return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
		}

		if err = s.publishUpdateEvent(ctx, eTC); err != nil {
			return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
		}
	}

	// Always update prevTc
	if err = s.updateTreeClusterPosition(ctx, prevTc.ID); err != nil {
		log.Error("error while update the cluster locations", "error", err, "cluster_id", prevTc.ID)
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}
	if err = s.publishUpdateEvent(ctx, prevTc); err != nil {
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	return s.GetByID(ctx, id)
}

func (s *TreeClusterService) Delete(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)
	_, err := s.treeClusterRepo.GetByID(ctx, id)
	if err != nil {
		return ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	if err := s.treeRepo.UnlinkTreeClusterID(ctx, id); err != nil {
		log.Debug("failed to unlink tree from tree cluster", "cluster_id", id, "error", err)
		return ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	if err := s.treeClusterRepo.Delete(ctx, id); err != nil {
		log.Debug("failed to delete tree cluster", "error", err, "cluster_id", id)
		return ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	log.Info("tree cluster deleted successfully", "cluster_id", id)
	return nil
}

func (s *TreeClusterService) UpdateWateringStatuses(ctx context.Context) error {
	log := logger.GetLogger(ctx)
	treeClusters, _, err := s.treeClusterRepo.GetAll(ctx, cluster.TreeClusterQuery{})
	if err != nil {
		log.Error("failed to fetch tree cluster", "error", err)
		return err
	}

	cutoffTime := time.Now().Add(-24 * time.Hour) // 1 day ago
	for _, tc := range treeClusters {
		var wateringStatus shared.WateringStatus

		if len(tc.TreeIDs) == 0 {
			wateringStatus = shared.WateringStatusUnknown
		} else if tc.LastWatered != nil && tc.LastWatered.Before(cutoffTime) {
			wateringStatus, err = s.getWateringStatusOfTreeCluster(ctx, tc.ID)
			if err != nil {
				log.Error("failed to get watering status of cluster", "cluster_id", tc.ID, "error", err)
				return err
			}
		}

		if wateringStatus != "" {
			err = s.treeClusterRepo.Update(ctx, tc.ID, func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
				tc.WateringStatus = wateringStatus
				return true, nil
			})
			if err != nil {
				log.Error("failed to update watering status of tree cluster", "cluster_id", tc.ID, "error", err)
			} else {
				log.Debug("watering status of tree cluster is updated", "cluster_id", tc.ID)
			}
		}
	}

	log.Info("watering status update for tree clusters completed successfully")
	return nil
}

func (s *TreeClusterService) Ready() bool {
	return s.treeClusterRepo != nil
}

func (s *TreeClusterService) updateTreeClusterPosition(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)
	err := s.treeClusterRepo.Update(ctx, id, func(tc *cluster.TreeCluster, repo cluster.TreeClusterRepository) (bool, error) {
		if len(tc.TreeIDs) != 0 {
			coord, err := repo.GetCenterPoint(ctx, tc.ID)
			if err != nil {
				log.Error("failed to get center point of tree cluster", "error", err, "tree_cluster", tc)
				return false, err
			}

			rgn, err := s.regionRepo.GetByPoint(ctx, *coord)
			if err != nil {
				log.Error("can't find region by coordinate", "error", err, "coordinate", coord, "tree_cluster", tc)
				return false, err
			}

			if tc.RegionID == nil && rgn != nil {
				tc.RegionID = &rgn.ID
			}

			if rgn != nil && tc.RegionID != nil && *tc.RegionID != rgn.ID {
				tc.RegionID = &rgn.ID
				log.Debug("updating region in tree cluster position", "id", rgn.ID, "name", rgn.Name)
			}

			if tc.Coordinate == nil || *tc.Coordinate != *coord {
				tc.Coordinate = coord

				wateringStatus, err := s.getWateringStatusOfTreeCluster(ctx, tc.ID)
				if err != nil {
					log.Error("could not calculate watering status", "error", err)
				} else {
					tc.WateringStatus = wateringStatus
				}

				log.Info("update tree cluster position due to changed trees inside the tree cluster", "cluster_id", id)
				log.Debug("detailed updated tree cluster position informations", "cluster_id", id,
					slog.Group("new_position", "latitude", coord.Latitude(), "longitude", coord.Longitude()),
				)
			}
		}

		return true, nil
	})

	return err
}

func (s *TreeClusterService) handlePrevTreeLocation(ctx context.Context, trees []*tree.Tree, updateFn func(context.Context, int32, func(*cluster.TreeCluster, cluster.TreeClusterRepository) (bool, error)) error) error {
	log := logger.GetLogger(ctx)
	visitedClusters := make(map[int32]bool)
	for _, t := range trees {
		if t.TreeClusterID == nil || *t.TreeClusterID == 0 {
			continue
		}

		tcID := *t.TreeClusterID
		if _, ok := visitedClusters[tcID]; ok {
			continue
		}

		updateFunc := func(tc *cluster.TreeCluster, repo cluster.TreeClusterRepository) (bool, error) {
			if len(tc.TreeIDs) != 0 {
				coord, err := repo.GetCenterPoint(ctx, tc.ID)
				if err != nil {
					log.Error("failed to get center point of tree cluster", "error", err, "tree_cluster", tc)
					return false, err
				}

				rgn, err := s.regionRepo.GetByPoint(ctx, *coord)
				if err != nil {
					log.Error("can't find region by coordinate", "error", err, "coordinate", coord, "tree_cluster", tc)
					return false, err
				}

				tc.Coordinate = coord
				if rgn != nil {
					tc.RegionID = &rgn.ID
				}
			}

			return true, nil
		}

		if err := updateFn(ctx, tcID, updateFunc); err != nil {
			log.Error("failed to update tree cluster after handling prev tree locations", "error", err, "cluster_id", tcID, "tree_id", t.ID)
			return err
		}

		prevTc, err := s.treeClusterRepo.GetByID(ctx, tcID)
		if err == nil {
			if err := s.publishUpdateEvent(ctx, prevTc); err != nil {
				return err
			}
		}

		visitedClusters[tcID] = true
	}

	log.Info("successfully updated tree cluster locations from prev trees",
		"tree_ids", utils.Map(trees, func(t *tree.Tree) int32 { return t.ID }),
		"updated_clusters", utils.MapKeysSlice(visitedClusters, func(k int32, _ bool) int32 { return k }),
	)
	return nil
}

func (s *TreeClusterService) getTrees(ctx context.Context, ids []*int32) ([]*tree.Tree, error) {
	treeIDs := make([]int32, len(ids))
	for i, id := range ids {
		treeIDs[i] = *id
	}

	return s.treeRepo.GetTreesByIDs(ctx, treeIDs)
}

func treeIDsFromTrees(trees []*tree.Tree) []int32 {
	ids := make([]int32, len(trees))
	for i, t := range trees {
		ids[i] = t.ID
	}
	return ids
}
