package cluster

import (
	"context"
	"errors"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

// HandleNewSensorData processes new sensor data and updates the watering status of a tree cluster if necessary.
func (s *TreeClusterService) HandleNewSensorData(ctx context.Context, event *sensor.EventNewData) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeClusterService")
	tree, err := s.treeRepo.GetBySensorID(ctx, event.New.SensorID)
	if err != nil {
		if errors.Is(err, sensor.ErrNotFound) {
			log.Error("failed to get sensor by id", "sensor_id", event.New.SensorID, "err", err)
			return nil
		}
		log.Info("the sensor has no selected tree. This event will be ignored", "sensor_id", event.New.SensorID, "error", err)
		return nil
	}

	if tree.TreeClusterID == nil {
		log.Info("this tree has no linked tree cluster. This event will be ignored", "tree_id", tree.ID)
		return nil
	}

	tcID := *tree.TreeClusterID
	tc, err := s.treeClusterRepo.GetByID(ctx, tcID)
	if err != nil {
		log.Error("failed to get tree cluster", "cluster_id", tcID, "err", err)
		return nil
	}

	wateringStatus, err := s.getWateringStatusOfTreeCluster(ctx, tcID)
	if err != nil {
		log.Error("error while calculating watering status of tree cluster", "error", err)
		return nil
	}

	if wateringStatus == tc.WateringStatus {
		log.Debug("watering status has not changed", "watering_status", wateringStatus)
		return nil
	}

	tc.WateringStatus = wateringStatus
	if err := s.treeClusterRepo.Update(ctx, tcID, tc); err == nil {
		return s.publishUpdateEvent(ctx, tc)
	}

	return nil
}

func (s *TreeClusterService) getWateringStatusOfTreeCluster(ctx context.Context, tcID int32) (shared.WateringStatus, error) {
	log := logger.GetLogger(ctx)
	sensorData, err := s.treeClusterRepo.GetAllLatestSensorDataByClusterID(ctx, tcID)
	if err != nil {
		log.Error("failed to get latest sensor data", "cluster_id", tcID, "err", err)
		return shared.WateringStatusUnknown, errors.New("failed to get latest sensor data")
	}

	trees, err := s.treeRepo.GetByTreeClusterID(ctx, tcID)
	if err != nil {
		log.Error("failed to get trees for cluster", "cluster_id", tcID, "err", err)
		return shared.WateringStatusUnknown, errors.New("failed to get trees for cluster")
	}

	if len(trees) == 0 || len(sensorData) == 0 {
		return shared.WateringStatusUnknown, nil
	}

	// Find the youngest tree (highest planting year)
	youngestTree := trees[0]
	for _, t := range trees[1:] {
		if t.PlantingYear.Year() > youngestTree.PlantingYear.Year() {
			youngestTree = t
		}
	}

	// Calculate average watermarks from sensor data
	avgWatermarks := averageWatermarks(sensorData)

	return youngestTree.CalculateWateringStatus(avgWatermarks)
}

func averageWatermarks(sensorData []*sensor.SensorData) []sensor.Watermark {
	type depthAccum struct {
		centibar   int
		resistance int
		count      int
	}
	depthMap := make(map[int]*depthAccum)

	for _, sd := range sensorData {
		if sd.Data == nil {
			continue
		}
		for _, wm := range sd.Data.Watermarks {
			acc, ok := depthMap[wm.Depth]
			if !ok {
				acc = &depthAccum{}
				depthMap[wm.Depth] = acc
			}
			acc.centibar += wm.Centibar
			acc.resistance += wm.Resistance
			acc.count++
		}
	}

	result := make([]sensor.Watermark, 0, len(depthMap))
	for depth, acc := range depthMap {
		if acc.count == 0 {
			continue
		}
		result = append(result, sensor.Watermark{
			Depth:      depth,
			Centibar:   acc.centibar / acc.count,
			Resistance: acc.resistance / acc.count,
		})
	}

	return result
}
