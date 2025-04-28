package sensor

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/go-playground/validator/v10"
	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/backend/internal/worker"
)

type SensorService struct {
	sensorRepo   storage.SensorRepository
	treeRepo     storage.TreeRepository
	validator    *validator.Validate
	eventManager *worker.EventManager
}

func NewSensorService(
	sensorRepo storage.SensorRepository,
	treeRepo storage.TreeRepository,
	eventManager *worker.EventManager,
) service.SensorService {
	return &SensorService{
		sensorRepo:   sensorRepo,
		treeRepo:     treeRepo,
		validator:    validator.New(),
		eventManager: eventManager,
	}
}

func (s *SensorService) publishNewSensorDataEvent(ctx context.Context, data *entities.SensorData) {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", entities.EventTypeNewSensorData, "service", "SensorService")
	event := entities.NewEventSensorData(data)
	if err := s.eventManager.Publish(ctx, event); err != nil {
		log.Error("error while sending event after new sensor data received", "err", err)
	}
}

func (s *SensorService) GetAll(ctx context.Context, query entities.Query) ([]*entities.Sensor, int64, error) {
	log := logger.GetLogger(ctx)
	sensors, totalCount, err := s.sensorRepo.GetAll(ctx, query)

	if err != nil {
		log.Debug("failed to fetch sensors", "error", err)
		return nil, 0, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	return sensors, totalCount, nil
}

func (s *SensorService) GetAllDataByID(ctx context.Context, id string) ([]*entities.SensorData, error) {
	log := logger.GetLogger(ctx)
	sensorData, err := s.sensorRepo.GetAllDataByID(ctx, id)

	if err != nil {
		log.Debug("failed to fetch sensor data", "error", err)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	return sensorData, nil
}

func (s *SensorService) GetByID(ctx context.Context, id string) (*entities.Sensor, error) {
	log := logger.GetLogger(ctx)
	get, err := s.sensorRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to fetch sensor by id", "sensor_id", id, "error", err)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	return get, nil
}

func (s *SensorService) Create(ctx context.Context, sc *entities.SensorCreate) (*entities.Sensor, error) {
	log := logger.GetLogger(ctx)
	if err := s.validator.Struct(sc); err != nil {
		log.Debug("failed to validate sensor struct to create", "error", err, "raw_sensor", fmt.Sprintf("%+v", sc))
		return nil, service.MapError(ctx, errors.Join(err, service.ErrValidation), service.ErrorLogValidation)
	}

	created, err := s.sensorRepo.Create(ctx, func(s *entities.Sensor, _ storage.SensorRepository) (bool, error) {
		s.LatestData = sc.LatestData
		s.Status = sc.Status
		s.Provider = sc.Provider
		s.AdditionalInfo = sc.AdditionalInfo
		return true, nil
	})

	if err != nil {
		log.Debug("failed to create sensor", "error", err, "sensor_id", sc.ID)
		return nil, service.MapError(ctx, err, service.ErrorLogAll)
	}

	log.Info("sensor created successfully", "sensor_id", created.ID)
	return created, nil
}

func (s *SensorService) Update(ctx context.Context, id string, su *entities.SensorUpdate) (*entities.Sensor, error) {
	log := logger.GetLogger(ctx)
	if err := s.validator.Struct(su); err != nil {
		log.Debug("failed to validate sensor struct to update", "error", err, "raw_sensor", fmt.Sprintf("%+v", su))
		return nil, service.MapError(ctx, errors.Join(err, service.ErrValidation), service.ErrorLogValidation)
	}

	_, err := s.sensorRepo.GetByID(ctx, id)
	if err != nil {
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	updated, err := s.sensorRepo.Update(ctx, id, func(s *entities.Sensor, _ storage.SensorRepository) (bool, error) {
		s.LatestData = su.LatestData
		s.Status = su.Status
		s.Provider = su.Provider
		s.AdditionalInfo = su.AdditionalInfo
		return true, nil
	})

	if err != nil {
		log.Debug("failed to update sensor", "sensor_id", id, "error", err)
		return nil, service.MapError(ctx, err, service.ErrorLogAll)
	}

	log.Info("sensor updated successfully", "sensor_id", id)
	return updated, nil
}

func (s *SensorService) Delete(ctx context.Context, id string) error {
	log := logger.GetLogger(ctx)
	_, err := s.sensorRepo.GetByID(ctx, id)
	if err != nil {
		return service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	err = s.treeRepo.UnlinkSensorID(ctx, id)
	if err != nil {
		log.Debug("failed to unlink sensor from tree", "error", err, "sensor_id", id)
		return service.MapError(ctx, err, service.ErrorLogAll)
	}

	err = s.sensorRepo.Delete(ctx, id)
	if err != nil {
		log.Debug("failed to delete sensor", "error", err, "sensor_id", id)
		return service.MapError(ctx, err, service.ErrorLogAll)
	}

	return nil
}

func (s *SensorService) UpdateStatuses(ctx context.Context) error {
	log := logger.GetLogger(ctx)
	sensors, _, err := s.sensorRepo.GetAll(ctx, entities.Query{})
	if err != nil {
		log.Error("failed to fetch sensors", "error", err)
		return err
	}

	cutoffTime := time.Now().Add(-72 * time.Hour) // 3 days ago
	for _, sens := range sensors {
		sensorData, err := s.sensorRepo.GetLatestSensorDataBySensorID(ctx, sens.ID)
		if err != nil {
			log.Error("failed to fetch latest sensor data", "sensor_id", sens.ID, "error", err)
			continue
		}
		if sensorData.CreatedAt.Before(cutoffTime) {
			_, err = s.sensorRepo.Update(ctx, sens.ID, func(s *entities.Sensor, _ storage.SensorRepository) (bool, error) {
				s.Status = entities.SensorStatusOffline
				return true, nil
			})

			if err != nil {
				log.Error("failed to update sensor status to offline", "sensor_id", sens.ID, "error", err, "prev_sensor_status", sens.Status)
			} else {
				log.Debug("sensor marked as offline due to inactivity", "sensor_id", sens.ID, "prev_sensor_status", sens.Status)
			}
		}
	}

	log.Info("sensor status update process completed successfully")
	return nil
}

func (s *SensorService) MapSensorToTree(ctx context.Context, sen *entities.Sensor) error {
	log := logger.GetLogger(ctx)
	if sen == nil {
		return errors.New("sensor cannot be nil")
	}

	nearestTree, err := s.treeRepo.FindNearestTree(ctx, sen.Latitude, sen.Longitude)
	if err != nil {
		log.Error("failed to calculate nearest tree", "sensor_id", sen.ID, "sensor_latitude", sen.Latitude, "sensor_longitude", sen.Longitude)
		return err
	}

	if nearestTree != nil {
		_, err = s.treeRepo.Update(ctx, nearestTree.ID, func(tree *entities.Tree, _ storage.TreeRepository) (bool, error) {
			tree.Sensor = sen
			log.Debug("update sensor on tree", "tree_id", tree.ID, "sensor_id", sen.ID)
			return true, nil
		})
		if err != nil {
			log.Error("failed to link sensor to nearest calculated tree", "tree_id", nearestTree.ID, "sensor_id", sen.ID, "error", err)
			return err
		}
	}

	return nil
}

func (s *SensorService) Ready() bool {
	return s.sensorRepo != nil
}
