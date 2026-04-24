package sensor

import (
	"context"
	"encoding/json"

	sensorDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"

	"github.com/pkg/errors"
)

func (r *SensorRepository) Create(ctx context.Context, entity *sensorDomain.Sensor) (*sensorDomain.Sensor, error) {
	log := logger.GetLogger(ctx)
	if entity == nil {
		return nil, errors.New("entity is nil")
	}

	if err := r.validateSensorEntity(entity); err != nil {
		return nil, err
	}

	existingSensor, _ := r.GetByID(ctx, entity.ID)
	if existingSensor != nil {
		return nil, errors.New("sensor with same ID already exists")
	}

	var createdSensor *sensorDomain.Sensor
	err := r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewSensorRepository(s, r.SensorRepositoryMappers)

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

func (r *SensorRepository) InsertSensorData(ctx context.Context, latestData *sensorDomain.SensorData, id sensorDomain.SensorID) error {
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

func (r *SensorRepository) createEntity(ctx context.Context, sn *sensorDomain.Sensor) (sensorDomain.SensorID, error) {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(sn.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", sn.AdditionalInfo)
		return sensorDomain.SensorID{}, err
	}

	id, err := r.store.CreateSensor(ctx, &sqlc.CreateSensorParams{
		ID:                     sn.ID.String(),
		Status:                 sqlc.SensorStatus(sn.Status),
		Provider:               &sn.Provider,
		AdditionalInformations: additionalInfo,
	})
	if err != nil {
		return sensorDomain.SensorID{}, err
	}

	if err := r.store.SetSensorLocation(ctx, &sqlc.SetSensorLocationParams{
		ID:        id,
		Latitude:  sn.Coordinate.Latitude(),
		Longitude: sn.Coordinate.Longitude(),
	}); err != nil {
		return sensorDomain.SensorID{}, err
	}
	return sensorDomain.NewSensorID(id)
}

func (r *SensorRepository) validateSensorEntity(sn *sensorDomain.Sensor) error {
	if sn == nil {
		return errors.New("sensor is nil")
	}
	if sn.ID.String() == "" {
		return errors.New("sensor id cannot be empty")
	}
	return nil
}
