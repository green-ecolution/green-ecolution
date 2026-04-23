package sensor

import (
	"context"
	"encoding/json"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"

	"github.com/pkg/errors"

	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
)

func defaultSensor() *shared.Sensor {
	return &shared.Sensor{
		Status:         shared.SensorStatusUnknown,
		LatestData:     nil,
		Coordinate:     shared.MustNewCoordinate(0, 0),
		Provider:       "",
		AdditionalInfo: nil,
	}
}

func (r *SensorRepository) Create(ctx context.Context, createFn func(*shared.Sensor, shared.SensorRepository) (bool, error)) (*shared.Sensor, error) {
	log := logger.GetLogger(ctx)
	if createFn == nil {
		return nil, errors.New("createFn is nil")
	}

	var createdSensor *shared.Sensor
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
		log.Debug("sensor entity created successfully in db", "sensor_id", id.String())

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

func (r *SensorRepository) InsertSensorData(ctx context.Context, latestData *shared.SensorData, id shared.SensorID) error {
	log := logger.GetLogger(ctx)
	if latestData == nil || latestData.Data == nil {
		return errors.New("latest data cannot be empty")
	}

	mqttData := r.mapper.FromDomainSensorData(latestData.Data)
	raw, err := json.Marshal(mqttData)
	if err != nil {
		return errors.Wrap(err, "failed to marshal mqtt data")
	}

	params := &sqlc.InsertSensorDataParams{
		SensorID: id.String(),
		Data:     raw,
	}

	err = r.store.InsertSensorData(ctx, params)
	if err != nil {
		log.Error("failed to insert sensor data in db", "error", err, "sensor_id", id.String())
		return err
	}

	return nil
}

func (r *SensorRepository) createEntity(ctx context.Context, sensor *shared.Sensor) (shared.SensorID, error) {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(sensor.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", sensor.AdditionalInfo)
		return shared.SensorID{}, err
	}

	id, err := r.store.CreateSensor(ctx, &sqlc.CreateSensorParams{
		ID:                     sensor.ID.String(),
		Status:                 sqlc.SensorStatus(sensor.Status),
		Provider:               &sensor.Provider,
		AdditionalInformations: additionalInfo,
	})
	if err != nil {
		return shared.SensorID{}, err
	}

	if err := r.store.SetSensorLocation(ctx, &sqlc.SetSensorLocationParams{
		ID:        id,
		Latitude:  sensor.Coordinate.Latitude(),
		Longitude: sensor.Coordinate.Longitude(),
	}); err != nil {
		return shared.SensorID{}, err
	}
	return shared.NewSensorID(id)
}

func (r *SensorRepository) validateSensorEntity(sensor *shared.Sensor) error {
	if sensor == nil {
		return errors.New("sensor is nil")
	}
	if sensor.ID.String() == "" {
		return errors.New("sensor id cannot be empty")
	}
	return nil
}
