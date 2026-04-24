package sensor

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/go-playground/validator/v10"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	sensorDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	treeDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

type SensorService struct {
	sensorRepo   sensorDomain.SensorRepository
	treeRepo     treeDomain.TreeRepository
	validator    *validator.Validate
	eventManager *worker.EventManager
}

func NewSensorService(
	sensorRepo sensorDomain.SensorRepository,
	treeRepo treeDomain.TreeRepository,
	eventManager *worker.EventManager,
) ports.SensorService {
	return &SensorService{
		sensorRepo:   sensorRepo,
		treeRepo:     treeRepo,
		validator:    validator.New(),
		eventManager: eventManager,
	}
}

func (s *SensorService) publishNewSensorDataEvent(ctx context.Context, data *sensorDomain.SensorData) {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", sensorDomain.EventTypeNewData, "service", "SensorService")
	event := sensorDomain.NewEventNewData(data)
	if err := s.eventManager.Publish(ctx, event); err != nil {
		log.Error("error while sending event after new sensor data received", "err", err)
	}
}

func (s *SensorService) GetAll(ctx context.Context, query shared.Query) ([]*sensorDomain.Sensor, int64, error) {
	log := logger.GetLogger(ctx)
	sensors, totalCount, err := s.sensorRepo.GetAll(ctx, query)

	if err != nil {
		log.Debug("failed to fetch sensors", "error", err)
		return nil, 0, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return sensors, totalCount, nil
}

func (s *SensorService) GetAllDataByID(ctx context.Context, id sensorDomain.SensorID) ([]*sensorDomain.SensorData, error) {
	log := logger.GetLogger(ctx)
	sensorData, err := s.sensorRepo.GetAllDataByID(ctx, id)

	if err != nil {
		log.Debug("failed to fetch sensor data", "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return sensorData, nil
}

func (s *SensorService) GetByID(ctx context.Context, id sensorDomain.SensorID) (*sensorDomain.Sensor, error) {
	log := logger.GetLogger(ctx)
	get, err := s.sensorRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to fetch sensor by id", "sensor_id", id.String(), "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return get, nil
}

func (s *SensorService) Create(ctx context.Context, sc *sensorDomain.SensorCreate) (*sensorDomain.Sensor, error) {
	log := logger.GetLogger(ctx)
	if err := s.validator.Struct(sc); err != nil {
		log.Debug("failed to validate sensor struct to create", "error", err, "raw_sensor", fmt.Sprintf("%+v", sc))
		return nil, ports.MapError(ctx, errors.Join(err, ports.ErrValidation), ports.ErrorLogValidation)
	}

	if sc.ID == (sensorDomain.SensorID{}) {
		err := errors.Join(errors.New("sensor id must not be empty"), ports.ErrValidation)
		return nil, ports.MapError(ctx, err, ports.ErrorLogValidation)
	}

	if sc.Coordinate == (shared.Coordinate{}) {
		err := errors.Join(errors.New("coordinate must not be empty"), ports.ErrValidation)
		return nil, ports.MapError(ctx, err, ports.ErrorLogValidation)
	}

	entity := &sensorDomain.Sensor{
		ID:             sc.ID,
		Coordinate:     sc.Coordinate,
		LatestData:     sc.LatestData,
		Status:         sc.Status,
		Provider:       sc.Provider,
		AdditionalInfo: sc.AdditionalInfo,
	}
	created, err := s.sensorRepo.Create(ctx, entity)

	if err != nil {
		log.Debug("failed to create sensor", "error", err, "sensor_id", sc.ID.String())
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	log.Info("sensor created successfully", "sensor_id", created.ID.String())
	return created, nil
}

func (s *SensorService) Update(ctx context.Context, id sensorDomain.SensorID, su *sensorDomain.SensorUpdate) (*sensorDomain.Sensor, error) {
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

	entity := &sensorDomain.Sensor{
		Coordinate:     su.Coordinate,
		LatestData:     su.LatestData,
		Status:         su.Status,
		Provider:       su.Provider,
		AdditionalInfo: su.AdditionalInfo,
	}
	updated, err := s.sensorRepo.Update(ctx, id, entity)

	if err != nil {
		log.Debug("failed to update sensor", "sensor_id", id.String(), "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	log.Info("sensor updated successfully", "sensor_id", id.String())
	return updated, nil
}

func (s *SensorService) Delete(ctx context.Context, id sensorDomain.SensorID) error {
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
			sens.Status = sensorDomain.SensorStatusOffline
			_, err = s.sensorRepo.Update(ctx, sens.ID, sens)

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

func (s *SensorService) MapSensorToTree(ctx context.Context, sen *sensorDomain.Sensor) error {
	log := logger.GetLogger(ctx)
	if sen == nil {
		return errors.New("sensor cannot be nil")
	}

	nearestTrees, err := s.treeRepo.FindNearestTrees(ctx, sen.Coordinate, 3, 1)
	if err != nil {
		log.Error("failed to calculate nearest tree", "sensor_id", sen.ID.String(), "sensor_latitude", sen.Coordinate.Latitude(), "sensor_longitude", sen.Coordinate.Longitude())
		return err
	}

	if len(nearestTrees) > 0 {
		nearestTree := nearestTrees[0].Tree
		log.Debug("update sensor on tree", "tree_id", nearestTree.ID, "sensor_id", sen.ID.String())
		nearestTree.SensorID = &sen.ID
		_, err = s.treeRepo.Update(ctx, nearestTree.ID, nearestTree)
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
