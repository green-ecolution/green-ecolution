package vehicle

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
)

func TestVehicleRepository_UpdateSuite(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/vehicle")
	input := vehicle.Vehicle{
		Description:    "Updated description",
		NumberPlate:    "FL NEW 9876",
		WaterCapacity:  shared.MustNewWaterCapacity(10000),
		Type:           vehicle.VehicleTypeTransporter,
		Status:         vehicle.VehicleStatusAvailable,
		DrivingLicense: vehicle.DrivingLicenseB,
		Height:         2.75,
		Length:         6.0,
		Width:          5.0,
		Weight:         1.3,
		Model:          "New model 1615/17",
	}

	t.Run("should update vehicle", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input

		// when
		err := r.Update(context.Background(), 1, &entity)
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
		updateEntity := &vehicle.Vehicle{
			NumberPlate:    numberPlate,
			Height:         input.Height,
			Length:         input.Length,
			Width:          input.Width,
			Weight:         input.Weight,
			WaterCapacity:  input.WaterCapacity,
			Type:           input.Type,
			Status:         input.Status,
			DrivingLicense: input.DrivingLicense,
		}

		errFirst := r.Update(context.Background(), 1, updateEntity)
		firstVehicle, _ := r.GetByID(context.Background(), 1)

		assert.NoError(t, errFirst)
		assert.NotNil(t, firstVehicle)

		errSecond := r.Update(context.Background(), 2, updateEntity)

		assert.Error(t, errSecond)
		assert.Contains(t, errSecond.Error(), "violates unique constraint")
	})

	t.Run("should return error when update vehicle with zero water capacity", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input
		entity.WaterCapacity = shared.MustNewWaterCapacity(0)

		// when
		err := r.Update(context.Background(), 1, &entity)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "water capacity is required and can not be 0")
	})

	t.Run("should return error when update vehicle with no number plate", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input
		entity.NumberPlate = ""

		// when
		err := r.Update(context.Background(), 1, &entity)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "number plate is required")
	})

	t.Run("should return error when update vehicle with zero size measurement", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input
		entity.Height = 0
		entity.Length = 0
		entity.Width = 0
		entity.Weight = 0

		// when
		err := r.Update(context.Background(), 1, &entity)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "size measurements are required and can not be 0")
	})

	t.Run("should return error when update vehicle with wrong driving license", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input
		entity.DrivingLicense = ""

		// when
		err := r.Update(context.Background(), 1, &entity)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "driving license is required and should be either B, BE or C")
	})

	t.Run("should return error when update vehicle with negative id", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input

		// when
		err := r.Update(context.Background(), -1, &entity)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when update vehicle with zero id", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input

		// when
		err := r.Update(context.Background(), 0, &entity)
		// then
		assert.Error(t, err)
	})

	t.Run("should return error when update vehicle not found", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input

		// when
		err := r.Update(context.Background(), 99, &entity)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()
		entity := input

		// when
		err := r.Update(ctx, 1, &entity)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when entity is nil", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		// when
		err := r.Update(context.Background(), 1, nil)

		// then
		assert.Error(t, err)
	})
}
