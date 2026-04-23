package sensor

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/go-playground/validator/v10"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

type SensorService struct {
	sensorRepo   shared.SensorRepository
	treeRepo     shared.TreeRepository
	validator    *validator.Validate
	eventManager *worker.EventManager
}

func NewSensorService(
	sensorRepo shared.SensorRepository,
	treeRepo shared.TreeRepository,
	eventManager *worker.EventManager,
) ports.SensorService {
	return &SensorService{
		sensorRepo:   sensorRepo,
		treeRepo:     treeRepo,
		validator:    validator.New(),
		eventManager: eventManager,
	}
}

func (s *SensorService) publishNewSensorDataEvent(ctx context.Context, data *shared.SensorData) {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", shared.EventTypeNewSensorData, "service", "SensorService")
	event := shared.NewEventSensorData(data)
	if err := s.eventManager.Publish(ctx, event); err != nil {
		log.Error("error while sending event after new sensor data received", "err", err)
	}
}

func (s *SensorService) GetAll(ctx context.Context, query shared.Query) ([]*shared.Sensor, int64, error) {
	log := logger.GetLogger(ctx)
	sensors, totalCount, err := s.sensorRepo.GetAll(ctx, query)

	if err != nil {
		log.Debug("failed to fetch sensors", "error", err)
		return nil, 0, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return sensors, totalCount, nil
}

func (s *SensorService) GetAllDataByID(ctx context.Context, id shared.SensorID) ([]*shared.SensorData, error) {
	log := logger.GetLogger(ctx)
	sensorData, err := s.sensorRepo.GetAllDataByID(ctx, id)

	if err != nil {
		log.Debug("failed to fetch sensor data", "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return sensorData, nil
}

func (s *SensorService) GetByID(ctx context.Context, id shared.SensorID) (*shared.Sensor, error) {
	log := logger.GetLogger(ctx)
	get, err := s.sensorRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to fetch sensor by id", "sensor_id", id.String(), "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return get, nil
}

func (s *SensorService) Create(ctx context.Context, sc *shared.SensorCreate) (*shared.Sensor, error) {
	log := logger.GetLogger(ctx)
	if err := s.validator.Struct(sc); err != nil {
		log.Debug("failed to validate sensor struct to create", "error", err, "raw_sensor", fmt.Sprintf("%+v", sc))
		return nil, ports.MapError(ctx, errors.Join(err, ports.ErrValidation), ports.ErrorLogValidation)
	}

	if sc.ID == (shared.SensorID{}) {
		err := errors.Join(errors.New("sensor id must not be empty"), ports.ErrValidation)
		return nil, ports.MapError(ctx, err, ports.ErrorLogValidation)
	}

	if sc.Coordinate == (shared.Coordinate{}) {
		err := errors.Join(errors.New("coordinate must not be empty"), ports.ErrValidation)
		return nil, ports.MapError(ctx, err, ports.ErrorLogValidation)
	}

	created, err := s.sensorRepo.Create(ctx, func(s *shared.Sensor, _ shared.SensorRepository) (bool, error) {
		s.ID = sc.ID
		s.Coordinate = sc.Coordinate
		s.LatestData = sc.LatestData
		s.Status = sc.Status
		s.Provider = sc.Provider
		s.AdditionalInfo = sc.AdditionalInfo
		return true, nil
	})

	if err != nil {
		log.Debug("failed to create sensor", "error", err, "sensor_id", sc.ID.String())
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	log.Info("sensor created successfully", "sensor_id", created.ID.String())
	return created, nil
}

func (s *SensorService) Update(ctx context.Context, id shared.SensorID, su *shared.SensorUpdate) (*shared.Sensor, error) {
	log := logger.GetLogger(ctx)
	if err := s.validator.Struct(su); err != nil {
		log.Debug("failed to validate sensor struct to update", "error", err, "raw_sensor", fmt.Sprintf("%+v", su))
		return nil, ports.MapError(ctx, errors.Join(err, ports.ErrValidation), ports.ErrorLogValidation)
	}

	if su.Coordinate == (shared.Coordinate{}) {
		err := errors.Join(errors.New("coordinate must not be empty"), ports.ErrValidation)
		return nil, ports.MapError(ctx, err, ports.ErrorLogValidation)
	}

	_, err := s.sensorRepo.GetByID(ctx, id)
	if err != nil {
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	updated, err := s.sensorRepo.Update(ctx, id, func(s *shared.Sensor, _ shared.SensorRepository) (bool, error) {
		s.Coordinate = su.Coordinate
		s.LatestData = su.LatestData
		s.Status = su.Status
		s.Provider = su.Provider
		s.AdditionalInfo = su.AdditionalInfo
		return true, nil
	})

	if err != nil {
		log.Debug("failed to update sensor", "sensor_id", id.String(), "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	log.Info("sensor updated successfully", "sensor_id", id.String())
	return updated, nil
}

func (s *SensorService) Delete(ctx context.Context, id shared.SensorID) error {
	log := logger.GetLogger(ctx)
	_, err := s.sensorRepo.GetByID(ctx, id)
	if err != nil {
		return ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	err = s.treeRepo.UnlinkSensorID(ctx, id)
	if err != nil {
		log.Debug("failed to unlink sensor from tree", "error", err, "sensor_id", id.String())
		return ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	err = s.sensorRepo.Delete(ctx, id)
	if err != nil {
		log.Debug("failed to delete sensor", "error", err, "sensor_id", id.String())
		return ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	return nil
}

func (s *SensorService) UpdateStatuses(ctx context.Context) error {
	log := logger.GetLogger(ctx)
	sensors, _, err := s.sensorRepo.GetAll(ctx, shared.Query{})
	if err != nil {
		log.Error("failed to fetch sensors", "error", err)
		return err
	}

	cutoffTime := time.Now().Add(-72 * time.Hour) // 3 days ago
	for _, sens := range sensors {
		sensorData, err := s.sensorRepo.GetLatestSensorDataBySensorID(ctx, sens.ID)
		if err != nil {
			log.Error("failed to fetch latest sensor data", "sensor_id", sens.ID.String(), "error", err)
			continue
		}
		if sensorData.CreatedAt.Before(cutoffTime) {
			_, err = s.sensorRepo.Update(ctx, sens.ID, func(s *shared.Sensor, _ shared.SensorRepository) (bool, error) {
				s.Status = shared.SensorStatusOffline
				return true, nil
			})

			if err != nil {
				log.Error("failed to update sensor status to offline", "sensor_id", sens.ID.String(), "error", err, "prev_sensor_status", sens.Status)
			} else {
				log.Debug("sensor marked as offline due to inactivity", "sensor_id", sens.ID.String(), "prev_sensor_status", sens.Status)
			}
		}
	}

	log.Info("sensor status update process completed successfully")
	return nil
}

func (s *SensorService) MapSensorToTree(ctx context.Context, sen *shared.Sensor) error {
	log := logger.GetLogger(ctx)
	if sen == nil {
		return errors.New("sensor cannot be nil")
	}

	nearestTree, err := s.treeRepo.FindNearestTree(ctx, sen.Coordinate)
	if err != nil {
		log.Error("failed to calculate nearest tree", "sensor_id", sen.ID.String(), "sensor_latitude", sen.Coordinate.Latitude(), "sensor_longitude", sen.Coordinate.Longitude())
		return err
	}

	if nearestTree != nil {
		_, err = s.treeRepo.Update(ctx, nearestTree.ID, func(tree *shared.Tree, _ shared.TreeRepository) (bool, error) {
			tree.Sensor = sen
			log.Debug("update sensor on tree", "tree_id", tree.ID, "sensor_id", sen.ID.String())
			return true, nil
		})
		if err != nil {
			log.Error("failed to link sensor to nearest calculated tree", "tree_id", nearestTree.ID, "sensor_id", sen.ID.String(), "error", err)
			return err
		}
	}

	return nil
}

func (s *SensorService) Ready() bool {
	return s.sensorRepo != nil
}
