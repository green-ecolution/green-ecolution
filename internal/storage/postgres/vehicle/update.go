package vehicle

import (
	"context"
	"errors"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/storage"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	store "github.com/green-ecolution/backend/internal/storage/postgres/store"
	"github.com/green-ecolution/backend/internal/utils"
)

func (r *VehicleRepository) Update(ctx context.Context, id int32, updateFn func(*entities.Vehicle, storage.VehicleRepository) (bool, error)) error {
	log := logger.GetLogger(ctx)
	return r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewVehicleRepository(s, r.VehicleRepositoryMappers)
		vh, err := newRepo.GetByID(ctx, id)
		if err != nil {
			return err
		}

		if updateFn == nil {
			return errors.New("updateFn is nil")
		}

		updated, err := updateFn(vh, newRepo)
		if err != nil {
			return err
		}

		if !updated {
			return nil
		}

		if err := newRepo.validateVehicle(vh); err != nil {
			return err
		}

		if err := newRepo.updateEntity(ctx, vh); err != nil {
			log.Error("failed to update vehicle entity in db", "error", err, "vehicle_id", id)
			return err
		}

		log.Debug("vehicle entity updated successfully in db", "vehicle_id", id)
		return nil
	})
}

func (r *VehicleRepository) updateEntity(ctx context.Context, vehicle *entities.Vehicle) error {
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
		WaterCapacity:          vehicle.WaterCapacity,
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
