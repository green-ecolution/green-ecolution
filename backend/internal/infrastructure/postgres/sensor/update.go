package sensor

import (
	"context"
	"errors"

	sensorDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func (r *SensorRepository) Update(ctx context.Context, id sensorDomain.SensorID, entity *sensorDomain.Sensor) (*sensorDomain.Sensor, error) {
	log := logger.GetLogger(ctx)
	if entity == nil {
		return nil, errors.New("entity is nil")
	}

	var updatedSensor *sensorDomain.Sensor
	err := r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewSensorRepository(s, r.SensorRepositoryMappers)

		if _, err := newRepo.GetByID(ctx, id); err != nil {
			return err
		}

		entity.ID = id
		if err := newRepo.updateEntity(ctx, entity); err != nil {
			log.Error("failed to update sensor entity in db", "error", err, "sensor_id", id.String())
			return err
		}

		if entity.LatestData != nil && entity.LatestData.Data != nil {
			err := newRepo.InsertSensorData(ctx, entity.LatestData, entity.ID)
			if err != nil {
				return err
			}
		}

		var err error
		updatedSensor, err = newRepo.GetByID(ctx, entity.ID)
		if err != nil {
			return err
		}

		return nil
	})

	if err != nil {
		return nil, err
	}

	log.Debug("sensor entity updated successfully in db", "sensor_id", id.String())
	return updatedSensor, nil
}

func (r *SensorRepository) updateEntity(ctx context.Context, sn *sensorDomain.Sensor) error {
	log := logger.GetLogger(ctx)

	additionalInfo, err := utils.MapAdditionalInfoToByte(sn.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", sn.AdditionalInfo)
		return err
	}

	params := sqlc.UpdateSensorParams{
		ID:                     sn.ID.String(),
		Status:                 sqlc.SensorStatus(sn.Status),
		Provider:               &sn.Provider,
		AdditionalInformations: additionalInfo,
	}

	locationParams := &sqlc.SetSensorLocationParams{
		ID:        sn.ID.String(),
		Latitude:  sn.Coordinate.Latitude(),
		Longitude: sn.Coordinate.Longitude(),
	}

	if err := r.store.SetSensorLocation(ctx, locationParams); err != nil {
		return err
	}

	return r.store.UpdateSensor(ctx, &params)
}
