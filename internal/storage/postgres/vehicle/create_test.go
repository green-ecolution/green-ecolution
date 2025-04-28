package vehicle

import (
	"context"
	"testing"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/storage"
	"github.com/stretchr/testify/assert"
)

func TestVehicleRepository_Create(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
	input := entities.Vehicle{
		Description:    "Big car",
		WaterCapacity:  2000,
		Type:           entities.VehicleTypeTrailer,
		Status:         entities.VehicleStatusNotAvailable,
		DrivingLicense: entities.DrivingLicenseBE,
		Height:         1.5,
		Length:         2.0,
		Width:          2.0,
		Weight:         3.0,
		Model:          "1615/17 - Conrad - MAN TGE 3.180",
	}

	t.Run("should create vehicle", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		numberPlate := "FL ZU 9876"

		createFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.Description = input.Description
			vh.NumberPlate = numberPlate
			vh.Status = input.Status
			vh.Type = input.Type
			vh.Model = input.Model
			vh.DrivingLicense = input.DrivingLicense
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, input.Description, got.Description)
		assert.Equal(t, numberPlate, got.NumberPlate)
		assert.Equal(t, input.WaterCapacity, got.WaterCapacity)
		assert.Equal(t, input.Type, got.Type)
		assert.Equal(t, input.Status, got.Status)
		assert.Equal(t, input.DrivingLicense, got.DrivingLicense)
		assert.Equal(t, input.Height, got.Height)
		assert.Equal(t, input.Length, got.Length)
		assert.Equal(t, input.Width, got.Width)
		assert.Equal(t, input.Weight, got.Weight)
		assert.Equal(t, input.Model, got.Model)
	})

	t.Run("should create vehicle with no description, type, model, driving license and status", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		numberPlate := "FL ZB 9876"

		createFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = numberPlate
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, "", got.Description)
		assert.Equal(t, "", got.Model)
		assert.Equal(t, numberPlate, got.NumberPlate)
		assert.Equal(t, input.WaterCapacity, got.WaterCapacity)
		assert.Equal(t, entities.VehicleTypeUnknown, got.Type)
		assert.Equal(t, entities.VehicleStatusUnknown, got.Status)
		assert.Equal(t, entities.DrivingLicenseB, got.DrivingLicense)
		assert.Equal(t, input.Height, got.Height)
		assert.Equal(t, input.Length, got.Length)
		assert.Equal(t, input.Width, got.Width)
		assert.Equal(t, input.Weight, got.Weight)
	})

	t.Run("should return error when create vehicle with zero water capacity", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		createFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = input.NumberPlate
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = 0
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "water capacity is required and can not be 0")
		assert.Nil(t, got)
	})

	t.Run("should return error when create vehicle with no number plate", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		createFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = ""
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "number plate is required")
		assert.Nil(t, got)
	})

	t.Run("should return error when create vehicle with zero size measurements", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		numberPlate := "FL ZB 9876"

		createFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = numberPlate
			vh.Height = 0
			vh.Length = 0
			vh.Width = 0
			vh.Weight = 0
			vh.WaterCapacity = input.WaterCapacity
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "size measurements are required and can not be 0")
		assert.Nil(t, got)
	})

	t.Run("should return error when create vehicle with wrong driving license", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		createFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = input.NumberPlate
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			vh.DrivingLicense = "ABC"
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "number plate is required")
		assert.Nil(t, got)
	})

	t.Run("should return error when create vehicle with duplicate plate", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		numberPlate := "FL ZT 9876"

		createFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = numberPlate
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			vh.DrivingLicense = input.DrivingLicense
			return true, nil
		}
		firstVehicle, err := r.Create(context.Background(), createFn)

		// when
		assert.NoError(t, err)
		assert.NotNil(t, firstVehicle)

		secondVehicle, err := r.Create(context.Background(), createFn)

		assert.Error(t, err)
		assert.Contains(t, err.Error(), "violates unique constraint")
		assert.Nil(t, secondVehicle)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		createFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = input.NumberPlate
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			vh.DrivingLicense = input.DrivingLicense
			return true, nil
		}

		got, err := r.Create(ctx, createFn)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when createFn returns error", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		createFn := func(wp *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			return false, assert.AnError
		}

		wp, err := r.Create(context.Background(), createFn)
		assert.Error(t, err)
		assert.Nil(t, wp)
	})

	t.Run("should not create watering plan when createFn returns false", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		createFn := func(wp *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			return false, nil
		}

		// when
		wp, err := r.Create(context.Background(), createFn)

		// then
		assert.NoError(t, err)
		assert.Nil(t, wp)
	})

	t.Run("should rollback transaction when createFn returns false and not return error", func(t *testing.T) {
		// given
		newID := int32(9)

		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		createFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = input.NumberPlate
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			vh.DrivingLicense = input.DrivingLicense
			return false, nil
		}

		// when
		wp, err := r.Create(context.Background(), createFn)
		got, _ := suite.Store.GetWateringPlanByID(context.Background(), newID)

		// then
		assert.NoError(t, err)
		assert.Nil(t, wp)
		assert.Empty(t, got)
	})
}
