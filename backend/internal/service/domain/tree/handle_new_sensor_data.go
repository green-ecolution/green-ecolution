package tree

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/service/domain/utils"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
)

// HandleNewSensorData processes new sensor data and updates the tree's watering status if necessary.
//
// The function retrieves the tree associated with the given sensor ID, calculates the new watering status,
// and updates it if there is a change. If the status is updated, an update event is published.
//
// Parameters:
//   - ctx: The request context, enabling logging and tracing.
//   - event: Contains the new sensor data, including the sensor ID and measured watermarks.
//
// Returns:
//   - error: An error if updating the tree fails; otherwise, nil.
func (s *TreeService) HandleNewSensorData(ctx context.Context, event *entities.EventNewSensorData) error {
	log := logger.GetLogger(ctx)
	log.Debug("handle event", "event", event.Type(), "service", "TreeService")
	t, err := s.treeRepo.GetBySensorID(ctx, event.New.SensorID)
	if err != nil {
		log.Debug("failed to get tree by sensor id", "sensor_id", event.New.SensorID, "err", err)
		return nil
	}

	status := utils.CalculateWateringStatus(ctx, t.PlantingYear, event.New.Data.Watermarks)

	if status == t.WateringStatus {
		log.Debug("sensor status has not changed", "sensor_status", status)
		return nil
	}
	newTree, err := s.treeRepo.Update(ctx, t.ID, func(s *entities.Tree, _ storage.TreeRepository) (bool, error) {
		log.Debug("updating tree watering status", "prev_status", t.WateringStatus, "new_status", status)
		s.WateringStatus = status
		return true, nil
	})

	if err != nil {
		log.Error("failed to update tree with new watering status", "tree_id", t.ID, "watering_status", status, "err", err)
		return err
	}

	log.Info("watering status of tree has been successfully updated", "tree_id", t.ID, "prev_status", t.WateringStatus, "new_status", status)

	s.publishUpdateTreeEvent(ctx, t, newTree, nil)
	return nil
}
