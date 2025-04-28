package tree

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"time"

	"github.com/go-playground/validator/v10"
	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/backend/internal/service/domain/utils"
	"github.com/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/backend/internal/worker"
)

type TreeService struct {
	treeRepo        storage.TreeRepository
	sensorRepo      storage.SensorRepository
	treeClusterRepo storage.TreeClusterRepository
	validator       *validator.Validate
	eventManager    *worker.EventManager
}

func NewTreeService(
	repoTree storage.TreeRepository,
	repoSensor storage.SensorRepository,
	treeClusterRepo storage.TreeClusterRepository,
	eventManager *worker.EventManager,
) service.TreeService {
	return &TreeService{
		treeRepo:        repoTree,
		sensorRepo:      repoSensor,
		treeClusterRepo: treeClusterRepo,
		validator:       validator.New(),
		eventManager:    eventManager,
	}
}

func (s *TreeService) GetAll(ctx context.Context, query entities.TreeQuery) ([]*entities.Tree, int64, error) {
	log := logger.GetLogger(ctx)
	trees, totalCount, err := s.treeRepo.GetAll(ctx, query)
	if err != nil {
		log.Debug("failed to fetch trees", "error", err)
		return nil, 0, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	return trees, totalCount, nil
}

func (s *TreeService) GetByID(ctx context.Context, id int32) (*entities.Tree, error) {
	log := logger.GetLogger(ctx)
	tr, err := s.treeRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to fetch tree by id", "error", err, "tree_id", id)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	return tr, nil
}

func (s *TreeService) GetBySensorID(ctx context.Context, id string) (*entities.Tree, error) {
	log := logger.GetLogger(ctx)
	tr, err := s.treeRepo.GetBySensorID(ctx, id)
	if err != nil {
		log.Debug("failed to get tree by sensor id", "sensor_id", id, "error", err)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	return tr, nil
}

func (s *TreeService) publishUpdateTreeEvent(ctx context.Context, prevTree, updatedTree, prevTreeOfSensor *entities.Tree) {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", entities.EventTypeUpdateTree, "service", "TreeService")
	event := entities.NewEventUpdateTree(prevTree, updatedTree, prevTreeOfSensor)
	if err := s.eventManager.Publish(ctx, event); err != nil {
		log.Error("error while sending event after updating tree", "err", err, "tree_id", prevTree.ID)
	}
}

func (s *TreeService) publishCreateTreeEvent(ctx context.Context, newTree, prevTreeOfSensor *entities.Tree) {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", entities.EventTypeCreateTree, "service", "TreeService")
	event := entities.NewEventCreateTree(newTree, prevTreeOfSensor)
	if err := s.eventManager.Publish(ctx, event); err != nil {
		log.Error("error while sending event after creating tree", "err", err, "tree_id", newTree.ID)
	}
}

func (s *TreeService) publishDeleteTreeEvent(ctx context.Context, prevTree *entities.Tree) {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", entities.EventTypeDeleteTree, "service", "TreeService")
	event := entities.NewEventDeleteTree(prevTree)
	if err := s.eventManager.Publish(ctx, event); err != nil {
		log.Error("error while sending event after deleting tree", "err", err, "tree_id", prevTree.ID)
	}
}

func (s *TreeService) Create(ctx context.Context, treeCreate *entities.TreeCreate) (*entities.Tree, error) {
	log := logger.GetLogger(ctx)
	if err := s.validator.Struct(treeCreate); err != nil {
		log.Debug("failed to validate tree struct to create", "error", err, "raw_tree", fmt.Sprintf("%+v", treeCreate))
		return nil, service.MapError(ctx, errors.Join(err, service.ErrValidation), service.ErrorLogValidation)
	}

	var prevTreeOfSensor *entities.Tree
	newTree, err := s.treeRepo.Create(ctx, func(tree *entities.Tree, repo storage.TreeRepository) (bool, error) {
		tree.PlantingYear = treeCreate.PlantingYear
		tree.Species = treeCreate.Species
		tree.Number = treeCreate.Number
		tree.Latitude = treeCreate.Latitude
		tree.Longitude = treeCreate.Longitude
		tree.Provider = treeCreate.Provider
		tree.AdditionalInfo = treeCreate.AdditionalInfo

		if treeCreate.TreeClusterID != nil {
			var err error
			treeCluster, err := s.treeClusterRepo.GetByID(ctx, *treeCreate.TreeClusterID)
			if err != nil {
				log.Debug("failed to fetch tree cluster by id specified in the tree create request", "tree_cluster_id", treeCreate.TreeClusterID)
				return false, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
			}
			tree.TreeCluster = treeCluster
		}

		if treeCreate.SensorID != nil {
			sensor, err := s.sensorRepo.GetByID(ctx, *treeCreate.SensorID)
			if err != nil {
				log.Debug("failed to fetch sensor by id specified in the tree create request", "sensor_id", treeCreate.SensorID)
				return false, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
			}
			tree.Sensor = sensor
			prevTreeOfSensor, err = repo.GetBySensorID(ctx, sensor.ID)
			if err != nil {
				// If the previous tree that was linked to the sensor could not be found, the create process should still be continued.
				log.Debug("failed to find previous tree linked to sensor specified from create request", "sensor_id", treeCreate.SensorID)
			}
			if sensor.LatestData != nil && sensor.LatestData.Data != nil && len(sensor.LatestData.Data.Watermarks) > 0 {
				status := utils.CalculateWateringStatus(ctx, treeCreate.PlantingYear, sensor.LatestData.Data.Watermarks)
				tree.WateringStatus = status
			}
		}

		return true, nil
	})

	if err != nil {
		log.Debug("failed to create tree", "error", err)
		return nil, service.MapError(ctx, err, service.ErrorLogAll)
	}

	slog.Info("tree created successfully", "tree_id", newTree.ID)
	s.publishCreateTreeEvent(ctx, newTree, prevTreeOfSensor)
	return newTree, nil
}

func (s *TreeService) Delete(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)
	treeEntity, err := s.treeRepo.GetByID(ctx, id)
	if err != nil {
		return service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}
	if err := s.treeRepo.Delete(ctx, id); err != nil {
		log.Debug("failed to delete tree", "error", err, "tree_id", id)
		return service.MapError(ctx, err, service.ErrorLogAll)
	}

	slog.Info("tree deleted successfully", "tree_id", id)
	s.publishDeleteTreeEvent(ctx, treeEntity)
	return nil
}

func (s *TreeService) Update(ctx context.Context, id int32, tu *entities.TreeUpdate) (*entities.Tree, error) {
	log := logger.GetLogger(ctx)
	if err := s.validator.Struct(tu); err != nil {
		log.Debug("failed to validate struct from tree update", "error", err, "raw_tree", fmt.Sprintf("%+v", tu))
		return nil, service.MapError(ctx, errors.Join(err, service.ErrValidation), service.ErrorLogValidation)
	}

	prevTree, err := s.treeRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to get previouse existing tree", "tree_id", id)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	var prevTreeOfSensor *entities.Tree
	updatedTree, err := s.treeRepo.Update(ctx, id, func(tree *entities.Tree, repo storage.TreeRepository) (bool, error) {
		tree.PlantingYear = tu.PlantingYear
		tree.Species = tu.Species
		tree.Number = tu.Number
		tree.Latitude = tu.Latitude
		tree.Longitude = tu.Longitude
		tree.Description = tu.Description
		tree.Provider = tu.Provider
		tree.AdditionalInfo = tu.AdditionalInfo

		if tu.TreeClusterID != nil {
			treeCluster, err := s.treeClusterRepo.GetByID(ctx, *tu.TreeClusterID)
			if err != nil {
				log.Debug("failed to find tree cluster by id specified from update request", "tree_cluster_id", tu.TreeClusterID)
				return false, service.MapError(ctx, fmt.Errorf("failed to find TreeCluster with ID %d: %w", *tu.TreeClusterID, err), service.ErrorLogEntityNotFound)
			}
			tree.TreeCluster = treeCluster
		} else {
			tree.TreeCluster = nil
		}

		if tu.SensorID != nil {
			sensor, err := s.sensorRepo.GetByID(ctx, *tu.SensorID)
			if err != nil {
				log.Debug("failed to find sensor by id specified from update request", "sensor_id", tu.SensorID)
				return false, service.MapError(ctx, fmt.Errorf("failed to find Sensor with ID %v: %w", *tu.SensorID, err), service.ErrorLogEntityNotFound)
			}
			tree.Sensor = sensor

			prevTreeOfSensor, err = repo.GetBySensorID(ctx, sensor.ID)
			if err != nil {
				// If the previous tree that was linked to the sensor could not be found, the update process should still be continued.
				log.Debug("failed to find previous tree linked to sensor specified from update request", "sensor_id", tu.SensorID)
			}
			if sensor.LatestData != nil && sensor.LatestData.Data != nil && len(sensor.LatestData.Data.Watermarks) > 0 {
				status := utils.CalculateWateringStatus(ctx, tu.PlantingYear, sensor.LatestData.Data.Watermarks)
				tree.WateringStatus = status
			}
		} else {
			tree.Sensor = nil
			tree.WateringStatus = entities.WateringStatusUnknown
		}

		return true, nil
	})

	if err != nil {
		log.Debug("failed to update tree", "error", err, "tree_id", id)
		return nil, service.MapError(ctx, err, service.ErrorLogAll)
	}

	slog.Info("tree updated successfully", "tree_id", id)
	s.publishUpdateTreeEvent(ctx, prevTree, updatedTree, prevTreeOfSensor)
	return updatedTree, nil
}

func (s *TreeService) UpdateWateringStatuses(ctx context.Context) error {
	log := logger.GetLogger(ctx)
	trees, _, err := s.treeRepo.GetAll(ctx, entities.TreeQuery{})
	if err != nil {
		log.Error("failed to fetch trees", "error", err)
		return err
	}

	cutoffTime := time.Now().Add(-24 * time.Hour) // 1 day ago
	for _, tree := range trees {
		// Do nothing if watering status is not »just watered«
		if tree.WateringStatus != entities.WateringStatusJustWatered {
			continue
		}

		if tree.LastWatered.Before(cutoffTime) {
			wateringStatus := entities.WateringStatusUnknown

			if tree.Sensor != nil {
				wateringStatus = utils.CalculateWateringStatus(ctx, tree.PlantingYear, tree.Sensor.LatestData.Data.Watermarks)
			}
			_, err = s.treeRepo.Update(ctx, tree.ID, func(tr *entities.Tree, _ storage.TreeRepository) (bool, error) {
				tr.WateringStatus = wateringStatus
				return true, nil
			})

			if err != nil {
				log.Error("failed to update watering status of tree", "tree_id", tree.ID, "error", err)
			} else {
				log.Debug("watering status of tree is updated by scheduler", "tree_id", tree.ID)
			}
		}
	}

	log.Info("watering status update for tree completed successfully")
	return nil
}

func (s *TreeService) Ready() bool {
	return s.treeRepo != nil && s.sensorRepo != nil
}
