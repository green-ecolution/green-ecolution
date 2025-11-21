package vehicle

import (
	"context"
	"errors"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	store "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func defaultVehicle() *entities.Vehicle {
	return &entities.Vehicle{
		NumberPlate:    "",
		Description:    "",
		WaterCapacity:  0,
		Type:           entities.VehicleTypeUnknown,
		Status:         entities.VehicleStatusUnknown,
		Model:          "",
		DrivingLicense: entities.DrivingLicenseB,
		Height:         0,
		Length:         0,
		Width:          0,
		Weight:         0,
		Provider:       "",
		AdditionalInfo: nil,
	}
}

func (r *VehicleRepository) Create(ctx context.Context, createFn func(*entities.Vehicle, storage.VehicleRepository) (bool, error)) (*entities.Vehicle, error) {
	log := logger.GetLogger(ctx)
	if createFn == nil {
		return nil, errors.New("createFn is nil")
	}

	var createdVh *entities.Vehicle
	err := r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewVehicleRepository(s, r.VehicleRepositoryMappers)
		entity := defaultVehicle()
		created, err := createFn(entity, newRepo)
		if err != nil {
			return err
		}

		if !created {
			return nil
		}

		if err := newRepo.validateVehicle(entity); err != nil {
			return err
		}

		id, err := newRepo.createEntity(ctx, entity)
		if err != nil {
			return err
		}
		createdVh, err = newRepo.GetByID(ctx, *id)
		if err != nil {
			return err
		}

		return nil
	})

	if err != nil {
		log.Error("failed to create vehicle entity in db", "error", err)
		return nil, err
	}

	if createdVh != nil {
		log.Debug("vehicle entity created successfully in db", "vehicle_id", createdVh.ID)
	}

	return createdVh, nil
}

func (r *VehicleRepository) createEntity(ctx context.Context, entity *entities.Vehicle) (*int32, error) {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(entity.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", entity.AdditionalInfo)
		return nil, err
	}

	args := sqlc.CreateVehicleParams{
		NumberPlate:            entity.NumberPlate,
		Description:            entity.Description,
		WaterCapacity:          entity.WaterCapacity,
		Type:                   sqlc.VehicleType(entity.Type),
		Status:                 sqlc.VehicleStatus(entity.Status),
		DrivingLicense:         sqlc.DrivingLicense(entity.DrivingLicense),
		Model:                  entity.Model,
		Width:                  entity.Width,
		Height:                 entity.Height,
		Length:                 entity.Length,
		Weight:                 entity.Weight,
		Provider:               &entity.Provider,
		AdditionalInformations: additionalInfo,
	}

	id, err := r.store.CreateVehicle(ctx, &args)
	if err != nil {
		return nil, err
	}

	return &id, nil
}

func (r *VehicleRepository) validateVehicle(entity *entities.Vehicle) error {
	if entity.WaterCapacity == 0 {
		return errors.New("water capacity is required and can not be 0")
	}

	if entity.Length == 0 || entity.Width == 0 || entity.Height == 0 || entity.Weight == 0 {
		return errors.New("size measurements are required and can not be 0")
	}

	if entity.NumberPlate == "" {
		return errors.New("number plate is required")
	}

	if entity.DrivingLicense == "" {
		return errors.New("driving license is required and should be either B, BE or C")
	}

	return nil
}
