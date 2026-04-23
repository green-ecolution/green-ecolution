package cluster

import (
	"context"
	"errors"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

// HandleNewSensorData processes new sensor data and updates the watering status of a tree cluster if necessary.
func (s *TreeClusterService) HandleNewSensorData(ctx context.Context, event *entities.EventNewSensorData) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeClusterService")
	tree, err := s.treeRepo.GetBySensorID(ctx, event.New.SensorID)
	if err != nil {
		if errors.Is(err, entities.ErrSensorNotFound) {
			log.Error("failed to get sensor by id", "sensor_id", event.New.SensorID, "err", err)
			return nil
		}
		log.Info("the sensor has no selected tree. This event will be ignored", "sensor_id", event.New.SensorID, "error", err)
		return nil
	}

	if tree.TreeCluster == nil {
		log.Info("this tree has no linked tree cluster. This event will be ignored", "tree_id", tree.ID)
		return nil
	}

	wateringStatus, err := s.getWateringStatusOfTreeCluster(ctx, tree.TreeCluster)
	if err != nil {
		log.Error("error while calculating watering status of tree cluster", "error", err)
		return nil
	}

	if wateringStatus == tree.TreeCluster.WateringStatus {
		log.Debug("watering status has not changed", "watering_status", wateringStatus)
		return nil
	}

	updateFn := func(tc *entities.TreeCluster, _ entities.TreeClusterRepository) (bool, error) {
		tc.WateringStatus = wateringStatus
		return true, nil
	}

	if err := s.treeClusterRepo.Update(ctx, tree.TreeCluster.ID, updateFn); err == nil {
		return s.publishUpdateEvent(ctx, tree.TreeCluster)
	}

	return nil
}

func (s *TreeClusterService) getWateringStatusOfTreeCluster(ctx context.Context, cluster *entities.TreeCluster) (entities.WateringStatus, error) {
	log := logger.GetLogger(ctx)
	sensorData, err := s.treeClusterRepo.GetAllLatestSensorDataByClusterID(ctx, cluster.ID)
	if err != nil {
		log.Error("failed to get latest sensor data", "cluster_id", cluster.ID, "err", err)
		return entities.WateringStatusUnknown, errors.New("failed to get latest sensor data")
	}

	return cluster.CalculateWateringStatus(sensorData)
}
