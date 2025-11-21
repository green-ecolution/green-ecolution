package sensor

import (
	"context"
	"errors"
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"

	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
)

func (r *SensorRepository) Update(ctx context.Context, id string, updateFn func(*entities.Sensor, storage.SensorRepository) (bool, error)) (*entities.Sensor, error) {
	log := logger.GetLogger(ctx)
	if updateFn == nil {
		return nil, errors.New("updateFn is nil")
	}

	var updatedSensor *entities.Sensor
	err := r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewSensorRepository(s, r.SensorRepositoryMappers)
		entity, err := newRepo.GetByID(ctx, id)
		if err != nil {
			return err
		}

		updated, err := updateFn(entity, newRepo)
		if err != nil {
			return err
		}

		if !updated {
			updatedSensor = entity
			return nil
		}

		if err := newRepo.updateEntity(ctx, entity); err != nil {
			log.Error("failed to update sensor entity in db", "error", err, "sensor_id", id)
			return err
		}

		if entity.LatestData != nil && entity.LatestData.Data != nil {
			err = newRepo.InsertSensorData(ctx, entity.LatestData, entity.ID)
			if err != nil {
				return err
			}
		}

		updatedSensor, err = newRepo.GetByID(ctx, entity.ID)
		if err != nil {
			return err
		}

		return nil
	})

	if err != nil {
		return nil, err
	}

	slog.Debug("sensor entity updated successfully in db", "sensor_id", id)
	return updatedSensor, nil
}

func (r *SensorRepository) updateEntity(ctx context.Context, sensor *entities.Sensor) error {
	log := logger.GetLogger(ctx)

	additionalInfo, err := utils.MapAdditionalInfoToByte(sensor.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", sensor.AdditionalInfo)
		return err
	}

	params := sqlc.UpdateSensorParams{
		ID:                     sensor.ID,
		Status:                 sqlc.SensorStatus(sensor.Status),
		Provider:               &sensor.Provider,
		AdditionalInformations: additionalInfo,
	}

	locationParams := &sqlc.SetSensorLocationParams{
		ID:        sensor.ID,
		Latitude:  sensor.Latitude,
		Longitude: sensor.Longitude,
	}

	if err := r.validateCoordinates(locationParams); err != nil {
		return err
	}
	if err := r.store.SetSensorLocation(ctx, locationParams); err != nil {
		return err
	}

	return r.store.UpdateSensor(ctx, &params)
}
func (r *SensorRepository) validateCoordinates(locationParams *sqlc.SetSensorLocationParams) error {
	if locationParams.Latitude < -90 || locationParams.Latitude > 90 || locationParams.Latitude == 0 {
		return storage.ErrInvalidLatitude
	}
	if locationParams.Longitude < -180 || locationParams.Longitude > 180 || locationParams.Longitude == 0 {
		return storage.ErrInvalidLongitude
	}

	return nil
}
