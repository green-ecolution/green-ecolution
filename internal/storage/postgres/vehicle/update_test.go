package vehicle

import (
	"context"
	"testing"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/storage"
	"github.com/stretchr/testify/assert"
)

func TestVehicleRepository_UpdateSuite(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
	input := entities.Vehicle{
		Description:    "Updated description",
		NumberPlate:    "FL NEW 9876",
		WaterCapacity:  10000,
		Type:           entities.VehicleTypeTransporter,
		Status:         entities.VehicleStatusAvailable,
		DrivingLicense: entities.DrivingLicenseB,
		Height:         2.75,
		Length:         6.0,
		Width:          5.0,
		Weight:         1.3,
		Model:          "New model 1615/17",
	}

	t.Run("should update vehicle", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		updateFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.Description = input.Description
			vh.NumberPlate = input.NumberPlate
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
		err := r.Update(context.Background(), 1, updateFn)
		got, _ := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, input.Description, got.Description)
		assert.Equal(t, input.NumberPlate, got.NumberPlate)
		assert.Equal(t, input.WaterCapacity, got.WaterCapacity)
		assert.Equal(t, input.Description, got.Description)
		assert.Equal(t, input.NumberPlate, got.NumberPlate)
		assert.Equal(t, input.WaterCapacity, got.WaterCapacity)
		assert.Equal(t, input.Type, got.Type)
		assert.Equal(t, input.Status, got.Status)
		assert.Equal(t, input.DrivingLicense, got.DrivingLicense)
		assert.Equal(t, input.Model, got.Model)
		assert.Equal(t, input.Height, got.Height)
		assert.Equal(t, input.Length, got.Length)
		assert.Equal(t, input.Width, got.Width)
		assert.Equal(t, input.Weight, got.Weight)
	})

	t.Run("should return error when update vehicle with duplicate plate", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		numberPlate := "FL ZT 9876"

		// when
		firstFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = numberPlate
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			return true, nil
		}

		errFirst := r.Update(context.Background(), 1, firstFn)
		firstVehicle, _ := r.GetByID(context.Background(), 1)

		assert.NoError(t, errFirst)
		assert.NotNil(t, firstVehicle)

		errSecond := r.Update(context.Background(), 2, firstFn)

		assert.Error(t, errSecond)
		assert.Contains(t, errSecond.Error(), "violates unique constraint")
	})

	t.Run("should return error when update vehicle with zero water capacity", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		updateFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = "FL ABC 123"
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = 0
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "water capacity is required and can not be 0")
	})

	t.Run("should return error when update vehicle with no number plate", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		updateFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = ""
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "number plate is required")
	})

	t.Run("should return error when update vehicle with zero size measurement", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		updateFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = "FL ABC 123"
			vh.Height = 0
			vh.Length = 0
			vh.Width = 0
			vh.Weight = 0
			vh.WaterCapacity = input.WaterCapacity
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "size measurements are required and can not be 0")
	})

	t.Run("should return error when update vehicle with wrong driving license", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		updateFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = "FL ABC 123"
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			vh.DrivingLicense = ""
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "driving license is required and should be either B, BE or C")
	})

	t.Run("should return error when update vehicle with negative id", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		updateFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = "FL ABC 123"
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			vh.DrivingLicense = "B"
			return true, nil
		}

		// when
		err := r.Update(context.Background(), -1, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when update vehicle with zero id", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		updateFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = "FL ABC 123"
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			vh.DrivingLicense = "B"
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 0, updateFn)
		// then
		assert.Error(t, err)
	})

	t.Run("should return error when update vehicle not found", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		updateFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = "FL ABC 123"
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			vh.DrivingLicense = "B"
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 99, updateFn)

		// then
		assert.Error(t, err)
		// assert.Equal(t, err, storage.ErrEntityNotFound)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		updateFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = "FL ABC 123"
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			vh.DrivingLicense = "B"
			return true, nil
		}

		// when
		err := r.Update(ctx, 1, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when updateFn is nil", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		// when
		err := r.Update(context.Background(), 1, nil)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when updateFn returns error", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		updateFn := func(wp *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			return true, assert.AnError
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should not update when updateFn returns false", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		updateFn := func(wp *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			return false, nil
		}

		// when
		updateErr := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
	})

	t.Run("should not rollback when updateFn returns false", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		updateFn := func(vh *entities.Vehicle, _ storage.VehicleRepository) (bool, error) {
			vh.NumberPlate = "FL ABC 123"
			vh.Height = input.Height
			vh.Length = input.Length
			vh.Width = input.Width
			vh.Weight = input.Weight
			vh.WaterCapacity = input.WaterCapacity
			vh.DrivingLicense = "B"
			return false, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, err)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotEqual(t, "FL ABC 123", got.NumberPlate)
	})
}
