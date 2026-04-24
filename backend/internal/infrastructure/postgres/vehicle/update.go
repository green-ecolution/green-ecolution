package vehicle

import (
	"context"
	"errors"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	store "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func (r *VehicleRepository) Update(ctx context.Context, id int32, entity *vehicle.Vehicle) error {
	log := logger.GetLogger(ctx)
	if entity == nil {
		return errors.New("entity is nil")
	}

	return r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewVehicleRepository(s, r.VehicleRepositoryMappers)

		if _, err := newRepo.GetByID(ctx, id); err != nil {
			return err
		}

		if err := newRepo.validateVehicle(entity); err != nil {
			return err
		}

		entity.ID = id
		if err := newRepo.updateEntity(ctx, entity); err != nil {
			log.Error("failed to update vehicle entity in db", "error", err, "vehicle_id", id)
			return err
		}

		log.Debug("vehicle entity updated successfully in db", "vehicle_id", id)
		return nil
	})
}

func (r *VehicleRepository) updateEntity(ctx context.Context, vehicle *vehicle.Vehicle) error {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(vehicle.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", vehicle.AdditionalInfo)
		return err
	}

	params := sqlc.UpdateVehicleParams{
		ID:                     vehicle.ID,
		NumberPlate:            vehicle.NumberPlate,
		Description:            vehicle.Description,
		WaterCapacity:          vehicle.WaterCapacity.Liters(),
		Type:                   sqlc.VehicleType(vehicle.Type),
		Status:                 sqlc.VehicleStatus(vehicle.Status),
		DrivingLicense:         sqlc.DrivingLicense(vehicle.DrivingLicense),
		Model:                  vehicle.Model,
		Height:                 vehicle.Height,
		Length:                 vehicle.Length,
		Width:                  vehicle.Width,
		Weight:                 vehicle.Weight,
		Provider:               &vehicle.Provider,
		AdditionalInformations: additionalInfo,
	}

	return r.store.UpdateVehicle(ctx, &params)
}
