package treecluster

import (
	"context"
	"errors"
	"slices"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	svcUtils "github.com/green-ecolution/green-ecolution/backend/internal/service/domain/utils"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

// HandleNewSensorData processes new sensor data and updates the watering status of a tree cluster if necessary.
//
// The function retrieves the tree associated with the given sensor ID and determines whether it belongs to a tree cluster.
// If the tree has a linked cluster, the watering status of the entire cluster is calculated based on the latest sensor data.
// If the calculated watering status differs from the current one, the tree cluster is updated accordingly, and an update event is published.
//
// Parameters:
//   - ctx: The request context, enabling logging and tracing.
//   - event: Contains the new sensor data, including the sensor ID and measured watermarks.
//
// Returns:
//   - error: Always returns nil, as errors during tree or cluster retrieval are logged but do not interrupt execution.
func (s *TreeClusterService) HandleNewSensorData(ctx context.Context, event *entities.EventNewSensorData) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeClusterService")
	tree, err := s.treeRepo.GetBySensorID(ctx, event.New.SensorID)
	if err != nil {
		// when error, it can be because the sensor has not linked tree or the tree does not exists
		if errors.Is(err, storage.ErrSensorNotFound) {
			log.Error("failed to get sensor by id", "sensor_id", event.New.SensorID, "err", err)
			return nil
		}
		log.Info("the sensor has no selected tree. This event will be ignored", "sensor_id", event.New.SensorID, "error", err)
		return nil
	}

	if tree.TreeCluster == nil {
		log.Info("this tree will has no linked tree cluster. This event will be ignored", "tree_id", tree.ID, "error", err)
		return nil
	}

	wateringStatus, err := s.getWateringStatusOfTreeCluster(ctx, tree.TreeCluster.ID)
	if err != nil {
		log.Error("error while calculating watering status of tree cluster", "error", err)
		return nil
	}

	if wateringStatus == tree.TreeCluster.WateringStatus {
		log.Debug("watering status has not changed", "watering_status", wateringStatus)
		return nil
	}

	updateFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
		tc.WateringStatus = wateringStatus
		return true, nil
	}

	if err := s.treeClusterRepo.Update(ctx, tree.TreeCluster.ID, updateFn); err == nil {
		return s.publishUpdateEvent(ctx, tree.TreeCluster)
	}

	return nil
}

func (s *TreeClusterService) getWateringStatusOfTreeCluster(ctx context.Context, clusterID int32) (entities.WateringStatus, error) {
	log := logger.GetLogger(ctx)
	sensorData, err := s.treeClusterRepo.GetAllLatestSensorDataByClusterID(ctx, clusterID)
	if err != nil {
		log.Error("failed to get latest sensor data", "cluster_id", clusterID, "err", err)
		return entities.WateringStatusUnknown, errors.New("failed to get latest sensor data")
	}

	// assertion - if there is no sensor data after receiving the event, the world is ending
	if len(sensorData) == 0 {
		log.Error("sensor data is empty")
		return entities.WateringStatusUnknown, errors.New("sensor data is empty")
	}

	sensorIDs := utils.Map(sensorData, func(data *entities.SensorData) string {
		return data.SensorID
	})

	youngestTree, err := s.getYoungestTree(ctx, sensorIDs)
	if err != nil {
		return entities.WateringStatusUnknown, errors.New("failed to get youngest tree")
	}

	watermarks, err := s.getWatermarkSensorData(ctx, sensorData)
	if err != nil {
		return entities.WateringStatusUnknown, errors.New("failed getting watermark sensor data")
	}

	return svcUtils.CalculateWateringStatus(ctx, youngestTree.PlantingYear, watermarks), nil
}

func (s *TreeClusterService) getYoungestTree(ctx context.Context, sensorIDs []string) (*entities.Tree, error) {
	log := logger.GetLogger(ctx)
	trees, err := s.treeRepo.GetBySensorIDs(ctx, sensorIDs...)
	if err != nil {
		log.Error("failed to get trees by sensor id", "sensor_ids", sensorIDs, "err", err)
		return nil, errors.New("failed to get trees by sensor id")
	}

	slices.SortFunc(trees, func(a, b *entities.Tree) int {
		return int(b.PlantingYear - a.PlantingYear)
	})

	return trees[0], nil
}

func (s *TreeClusterService) getWatermarkSensorData(ctx context.Context, sensorData []*entities.SensorData) ([]entities.Watermark, error) {
	log := logger.GetLogger(ctx)
	var w30CentibarAvg, w60CentibarAvg, w90CentibarAvg int
	for _, data := range sensorData {
		w30, w60, w90, err := svcUtils.CheckAndSortWatermarks(data.Data.Watermarks)
		if err != nil {
			log.Error("sensor data watermarks are malformed", "watermarks", data.Data.Watermarks)
			return nil, errors.New("sensor data watermarks are malformed")
		}

		w30CentibarAvg += w30.Centibar
		w60CentibarAvg += w60.Centibar
		w90CentibarAvg += w90.Centibar
	}

	return []entities.Watermark{
		{
			Centibar: w30CentibarAvg / len(sensorData),
			Depth:    30,
		},
		{
			Centibar: w60CentibarAvg / len(sensorData),
			Depth:    60,
		},
		{
			Centibar: w90CentibarAvg / len(sensorData),
			Depth:    90,
		},
	}, nil
}
