package tree

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	sensorDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	treeDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

type TreeService struct {
	treeRepo        treeDomain.TreeRepository
	sensorRepo      sensorDomain.SensorRepository
	treeClusterRepo cluster.TreeClusterRepository
	eventManager    *worker.EventManager
	mapCfg          config.MapConfig
}

func NewTreeService(
	repoTree treeDomain.TreeRepository,
	repoSensor sensorDomain.SensorRepository,
	treeClusterRepo cluster.TreeClusterRepository,
	eventManager *worker.EventManager,
	mapCfg config.MapConfig,
) ports.TreeService {
	return &TreeService{
		treeRepo:        repoTree,
		sensorRepo:      repoSensor,
		treeClusterRepo: treeClusterRepo,
		eventManager:    eventManager,
		mapCfg:          mapCfg,
	}
}

func (s *TreeService) GetAll(ctx context.Context, query treeDomain.TreeQuery) ([]*treeDomain.Tree, int64, error) {
	log := logger.GetLogger(ctx)
	trees, totalCount, err := s.treeRepo.GetAll(ctx, query)
	if err != nil {
		log.Debug("failed to fetch trees", "error", err)
		return nil, 0, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return trees, totalCount, nil
}

func (s *TreeService) GetByID(ctx context.Context, id int32) (*treeDomain.Tree, error) {
	log := logger.GetLogger(ctx)
	tr, err := s.treeRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to fetch tree by id", "error", err, "tree_id", id)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return tr, nil
}

func (s *TreeService) GetBySensorID(ctx context.Context, id sensorDomain.SensorID) (*treeDomain.Tree, error) {
	log := logger.GetLogger(ctx)
	tr, err := s.treeRepo.GetBySensorID(ctx, id)
	if err != nil {
		log.Debug("failed to get tree by sensor id", "sensor_id", id, "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return tr, nil
}

func (s *TreeService) GetNearestTrees(ctx context.Context, coord shared.Coordinate, limit int32) ([]*treeDomain.TreeWithDistance, error) {
	log := logger.GetLogger(ctx)

	if limit <= 0 {
		limit = int32(s.mapCfg.NearestTreeDefaultLimit)
	}
	if maxLimit := int32(s.mapCfg.NearestTreeMaxLimit); limit > maxLimit {
		limit = maxLimit
	}

	log.Debug("searching nearest trees", "lat", coord.Latitude(), "lng", coord.Longitude(), "radius", s.mapCfg.NearestTreeMaxRadius, "limit", limit)
	trees, err := s.treeRepo.FindNearestTrees(ctx, coord, s.mapCfg.NearestTreeMaxRadius, limit)
	if err != nil {
		log.Debug("failed to find nearest trees", "error", err, "lat", coord.Latitude(), "lng", coord.Longitude())
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}
	return trees, nil
}

func (s *TreeService) publishUpdateTreeEvent(ctx context.Context, prevTree, updatedTree, prevTreeOfSensor *treeDomain.Tree) {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", treeDomain.EventTypeUpdate, "service", "TreeService")
	event := treeDomain.NewEventUpdate(prevTree, updatedTree, prevTreeOfSensor)
	if err := s.eventManager.Publish(ctx, event); err != nil {
		log.Error("error while sending event after updating tree", "err", err, "tree_id", prevTree.ID)
	}
}

func (s *TreeService) publishCreateTreeEvent(ctx context.Context, newTree, prevTreeOfSensor *treeDomain.Tree) {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", treeDomain.EventTypeCreate, "service", "TreeService")
	event := treeDomain.NewEventCreate(newTree, prevTreeOfSensor)
	if err := s.eventManager.Publish(ctx, event); err != nil {
		log.Error("error while sending event after creating tree", "err", err, "tree_id", newTree.ID)
	}
}

func (s *TreeService) publishDeleteTreeEvent(ctx context.Context, prevTree *treeDomain.Tree) {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", treeDomain.EventTypeDelete, "service", "TreeService")
	event := treeDomain.NewEventDelete(prevTree)
	if err := s.eventManager.Publish(ctx, event); err != nil {
		log.Error("error while sending event after deleting tree", "err", err, "tree_id", prevTree.ID)
	}
}

func (s *TreeService) Create(ctx context.Context, treeCreate *treeDomain.TreeCreate) (*treeDomain.Tree, error) {
	log := logger.GetLogger(ctx)

	t := &treeDomain.Tree{
		PlantingYear:   treeCreate.PlantingYear,
		Species:        treeCreate.Species,
		Number:         treeCreate.Number,
		Coordinate:     treeCreate.Coordinate,
		Description:    treeCreate.Description,
		Provider:       treeCreate.Provider,
		AdditionalInfo: treeCreate.AdditionalInfo,
	}

	var prevTreeOfSensor *treeDomain.Tree

	if treeCreate.TreeClusterID != nil {
		_, err := s.treeClusterRepo.GetByID(ctx, *treeCreate.TreeClusterID)
		if err != nil {
			log.Debug("failed to fetch tree cluster by id specified in the tree create request", "tree_cluster_id", treeCreate.TreeClusterID)
			return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
		}
		t.TreeClusterID = treeCreate.TreeClusterID
	}

	if treeCreate.SensorID != nil {
		sensorEntity, err := s.sensorRepo.GetByID(ctx, *treeCreate.SensorID)
		if err != nil {
			log.Debug("failed to fetch sensor by id specified in the tree create request", "sensor_id", treeCreate.SensorID)
			return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
		}
		t.SensorID = treeCreate.SensorID
		prevTreeOfSensor, err = s.treeRepo.GetBySensorID(ctx, sensorEntity.ID)
		if err != nil {
			// If the previous tree that was linked to the sensor could not be found, the create process should still be continued.
			log.Debug("failed to find previous tree linked to sensor specified from create request", "sensor_id", treeCreate.SensorID)
		}
		if sensorEntity.LatestData != nil && sensorEntity.LatestData.Data != nil && len(sensorEntity.LatestData.Data.Watermarks) > 0 {
			status, err := t.CalculateWateringStatus(sensorEntity.LatestData.Data.Watermarks)
			if err != nil {
				return nil, err
			}
			t.WateringStatus = status
		}
	}

	newTree, err := s.treeRepo.Create(ctx, t)
	if err != nil {
		log.Debug("failed to create tree", "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	slog.Info("tree created successfully", "tree_id", newTree.ID)
	s.publishCreateTreeEvent(ctx, newTree, prevTreeOfSensor)
	return newTree, nil
}

func (s *TreeService) Delete(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)
	treeEntity, err := s.treeRepo.GetByID(ctx, id)
	if err != nil {
		return ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}
	if err := s.treeRepo.Delete(ctx, id); err != nil {
		log.Debug("failed to delete tree", "error", err, "tree_id", id)
		return ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	slog.Info("tree deleted successfully", "tree_id", id)
	s.publishDeleteTreeEvent(ctx, treeEntity)
	return nil
}

func (s *TreeService) Update(ctx context.Context, id int32, tu *treeDomain.TreeUpdate) (*treeDomain.Tree, error) {
	log := logger.GetLogger(ctx)

	prevTree, err := s.treeRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to get previouse existing tree", "tree_id", id)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	t := &treeDomain.Tree{
		PlantingYear:   tu.PlantingYear,
		Species:        tu.Species,
		Number:         tu.Number,
		Coordinate:     tu.Coordinate,
		Description:    tu.Description,
		Provider:       tu.Provider,
		AdditionalInfo: tu.AdditionalInfo,
	}

	var prevTreeOfSensor *treeDomain.Tree

	if tu.TreeClusterID != nil {
		_, err := s.treeClusterRepo.GetByID(ctx, *tu.TreeClusterID)
		if err != nil {
			log.Debug("failed to find tree cluster by id specified from update request", "tree_cluster_id", tu.TreeClusterID)
			return nil, ports.MapError(ctx, fmt.Errorf("failed to find TreeCluster with ID %d: %w", *tu.TreeClusterID, err), ports.ErrorLogEntityNotFound)
		}
		t.TreeClusterID = tu.TreeClusterID
	} else {
		t.TreeClusterID = nil
	}

	if tu.SensorID != nil {
		sensorEntity, err := s.sensorRepo.GetByID(ctx, *tu.SensorID)
		if err != nil {
			log.Debug("failed to find sensor by id specified from update request", "sensor_id", tu.SensorID)
			return nil, ports.MapError(ctx, fmt.Errorf("failed to find Sensor with ID %v: %w", *tu.SensorID, err), ports.ErrorLogEntityNotFound)
		}
		t.SensorID = tu.SensorID

		prevTreeOfSensor, err = s.treeRepo.GetBySensorID(ctx, sensorEntity.ID)
		if err != nil {
			// If the previous tree that was linked to the sensor could not be found, the update process should still be continued.
			log.Debug("failed to find previous tree linked to sensor specified from update request", "sensor_id", tu.SensorID)
		}
		if sensorEntity.LatestData != nil && sensorEntity.LatestData.Data != nil && len(sensorEntity.LatestData.Data.Watermarks) > 0 {
			status, err := t.CalculateWateringStatus(sensorEntity.LatestData.Data.Watermarks)
			if err != nil {
				return nil, err
			}
			t.WateringStatus = status
		}
	} else {
		t.RemoveSensor()
	}

	updatedTree, err := s.treeRepo.Update(ctx, id, t)
	if err != nil {
		log.Debug("failed to update tree", "error", err, "tree_id", id)
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	slog.Info("tree updated successfully", "tree_id", id)
	s.publishUpdateTreeEvent(ctx, prevTree, updatedTree, prevTreeOfSensor)
	return updatedTree, nil
}

func (s *TreeService) UpdateWateringStatuses(ctx context.Context) error {
	log := logger.GetLogger(ctx)
	trees, _, err := s.treeRepo.GetAll(ctx, treeDomain.TreeQuery{})
	if err != nil {
		log.Error("failed to fetch trees", "error", err)
		return err
	}

	cutoffTime := time.Now().Add(-24 * time.Hour) // 1 day ago
	for _, t := range trees {
		if !t.IsWateringStatusExpired(cutoffTime) {
			continue
		}

		if t.SensorID == nil {
			continue
		}

		sensorEntity, err := s.sensorRepo.GetByID(ctx, *t.SensorID)
		if err != nil {
			log.Debug("failed to fetch sensor for tree", "tree_id", t.ID, "sensor_id", *t.SensorID, "error", err)
			continue
		}
		if sensorEntity.LatestData == nil || sensorEntity.LatestData.Data == nil {
			continue
		}

		wateringStatus, hasChanged, err := t.RefreshWateringStatus(sensorEntity.LatestData.Data.Watermarks)
		if errors.Is(err, treeDomain.ErrNoSensorData) {
			continue
		}
		if err != nil {
			return err
		}

		if hasChanged {
			t.WateringStatus = wateringStatus
			_, err = s.treeRepo.Update(ctx, t.ID, t)
			if err != nil {
				log.Error("failed to update watering status of tree", "tree_id", t.ID, "error", err)
				return err
			}
		}
	}

	log.Info("watering status update for tree completed successfully")
	return nil
}

func (s *TreeService) GetPlantingYears(ctx context.Context) ([]int32, error) {
	log := logger.GetLogger(ctx)
	years, err := s.treeRepo.GetDistinctPlantingYears(ctx)
	if err != nil {
		log.Debug("failed to fetch planting years", "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}
	return years, nil
}

func (s *TreeService) Ready() bool {
	return s.treeRepo != nil && s.sensorRepo != nil
}
