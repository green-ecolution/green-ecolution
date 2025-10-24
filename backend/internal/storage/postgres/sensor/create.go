package sensor

import (
	"context"
	"encoding/json"

	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/storage/postgres/store"
	"github.com/green-ecolution/backend/internal/utils"

	"github.com/green-ecolution/backend/internal/storage"

	"github.com/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/pkg/errors"
)

func defaultSensor() *entities.Sensor {
	return &entities.Sensor{
		Status:         entities.SensorStatusUnknown,
		LatestData:     nil,
		Latitude:       0,
		Longitude:      0,
		Provider:       "",
		AdditionalInfo: nil,
	}
}

func (r *SensorRepository) Create(ctx context.Context, createFn func(*entities.Sensor, storage.SensorRepository) (bool, error)) (*entities.Sensor, error) {
	log := logger.GetLogger(ctx)
	if createFn == nil {
		return nil, errors.New("createFn is nil")
	}

	var createdSensor *entities.Sensor
	err := r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewSensorRepository(s, r.SensorRepositoryMappers)
		entity := defaultSensor()

		created, err := createFn(entity, newRepo)
		if err != nil {
			return err
		}

		if !created {
			return nil
		}

		existingSensor, _ := newRepo.GetByID(ctx, entity.ID)
		if existingSensor != nil {
			return errors.New("sensor with same ID already exists")
		}

		if err := newRepo.validateSensorEntity(entity); err != nil {
			return err
		}

		id, err := newRepo.createEntity(ctx, entity)
		if err != nil {
			log.Error("failed to create sensor entity in db", "error", err)
			return err
		}
		entity.ID = id
		log.Debug("sensor entity created successfully in db", "sensor_id", id)

		if entity.LatestData != nil && entity.LatestData.Data != nil {
			err = newRepo.InsertSensorData(ctx, entity.LatestData, id)
			if err != nil {
				return err
			}
		}

		createdSensor, err = newRepo.GetByID(ctx, id)
		if err != nil {
			return err
		}

		return nil
	})

	if err != nil {
		return nil, err
	}

	return createdSensor, nil
}

func (r *SensorRepository) InsertSensorData(ctx context.Context, latestData *entities.SensorData, id string) error {
	log := logger.GetLogger(ctx)
	if latestData == nil || latestData.Data == nil {
		return errors.New("latest data cannot be empty")
	}

	if id == "" {
		return errors.New("sensor id cannot be empty")
	}

	mqttData := r.mapper.FromDomainSensorData(latestData.Data)
	raw, err := json.Marshal(mqttData)
	if err != nil {
		return errors.Wrap(err, "failed to marshal mqtt data")
	}

	params := &sqlc.InsertSensorDataParams{
		SensorID: id,
		Data:     raw,
	}

	err = r.store.InsertSensorData(ctx, params)
	if err != nil {
		log.Error("failed to insert sensor data in db", "error", err, "sensor_id", id)
		return err
	}

	return nil
}

func (r *SensorRepository) createEntity(ctx context.Context, sensor *entities.Sensor) (string, error) {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(sensor.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", sensor.AdditionalInfo)
		return "", err
	}

	id, err := r.store.CreateSensor(ctx, &sqlc.CreateSensorParams{
		ID:                     sensor.ID,
		Status:                 sqlc.SensorStatus(sensor.Status),
		Provider:               &sensor.Provider,
		AdditionalInformations: additionalInfo,
	})
	if err != nil {
		return "", err
	}

	if err := r.store.SetSensorLocation(ctx, &sqlc.SetSensorLocationParams{
		ID:        id,
		Latitude:  sensor.Latitude,
		Longitude: sensor.Longitude,
	}); err != nil {
		return "", err
	}
	return id, nil
}

func (r *SensorRepository) validateSensorEntity(sensor *entities.Sensor) error {
	if sensor.ID == "" {
		return errors.New("sensor id cannot be empty")
	}
	if sensor.Latitude < -90 || sensor.Latitude > 90 {
		return storage.ErrInvalidLatitude
	}
	if sensor.Longitude < -180 || sensor.Longitude > 180 {
		return storage.ErrInvalidLongitude
	}

	return nil
}
