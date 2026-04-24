package vehicle

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
)

func TestVehicleRepository_Create(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/vehicle")
	input := vehicle.Vehicle{
		Description:    "Big car",
		WaterCapacity:  shared.MustNewWaterCapacity(2000),
		Type:           vehicle.VehicleTypeTrailer,
		Status:         vehicle.VehicleStatusNotAvailable,
		DrivingLicense: vehicle.DrivingLicenseBE,
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
		entity := input
		entity.NumberPlate = numberPlate

		// when
		got, err := r.Create(context.Background(), &entity)

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
		entity := vehicle.Vehicle{
			NumberPlate:    numberPlate,
			WaterCapacity:  input.WaterCapacity,
			Type:           vehicle.VehicleTypeUnknown,
			Status:         vehicle.VehicleStatusUnknown,
			DrivingLicense: vehicle.DrivingLicenseB,
			Height:         input.Height,
			Length:         input.Length,
			Width:          input.Width,
			Weight:         input.Weight,
		}

		// when
		got, err := r.Create(context.Background(), &entity)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, "", got.Description)
		assert.Equal(t, "", got.Model)
		assert.Equal(t, numberPlate, got.NumberPlate)
		assert.Equal(t, input.WaterCapacity, got.WaterCapacity)
		assert.Equal(t, vehicle.VehicleTypeUnknown, got.Type)
		assert.Equal(t, vehicle.VehicleStatusUnknown, got.Status)
		assert.Equal(t, vehicle.DrivingLicenseB, got.DrivingLicense)
		assert.Equal(t, input.Height, got.Height)
		assert.Equal(t, input.Length, got.Length)
		assert.Equal(t, input.Width, got.Width)
		assert.Equal(t, input.Weight, got.Weight)
	})

	t.Run("should return error when create vehicle with zero water capacity", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input
		entity.NumberPlate = "FL ZC 1111"
		entity.WaterCapacity = shared.MustNewWaterCapacity(0)

		// when
		got, err := r.Create(context.Background(), &entity)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "water capacity is required and can not be 0")
		assert.Nil(t, got)
	})

	t.Run("should return error when create vehicle with no number plate", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input
		entity.NumberPlate = ""

		// when
		got, err := r.Create(context.Background(), &entity)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "number plate is required")
		assert.Nil(t, got)
	})

	t.Run("should return error when create vehicle with zero size measurements", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input
		entity.NumberPlate = "FL ZD 1111"
		entity.Height = 0
		entity.Length = 0
		entity.Width = 0
		entity.Weight = 0

		// when
		got, err := r.Create(context.Background(), &entity)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "size measurements are required and can not be 0")
		assert.Nil(t, got)
	})

	t.Run("should return error when create vehicle with wrong driving license", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input
		entity.NumberPlate = ""
		entity.DrivingLicense = ""

		// when
		got, err := r.Create(context.Background(), &entity)

		// then
		assert.Error(t, err)
		assert.EqualError(t, err, "number plate is required")
		assert.Nil(t, got)
	})

	t.Run("should return error when create vehicle with duplicate plate", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		entity := input
		entity.NumberPlate = "FL ZE 9876"

		firstVehicle, err := r.Create(context.Background(), &entity)

		// when
		assert.NoError(t, err)
		assert.NotNil(t, firstVehicle)

		secondVehicle, err := r.Create(context.Background(), &entity)

		assert.Error(t, err)
		assert.Contains(t, err.Error(), "violates unique constraint")
		assert.Nil(t, secondVehicle)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		entity := input
		entity.NumberPlate = "FL ZF 9876"

		// when
		got, err := r.Create(ctx, &entity)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when entity is nil", func(t *testing.T) {
		// given
		r := NewVehicleRepository(defaultFields.store, defaultFields.VehicleMappers)

		wp, err := r.Create(context.Background(), nil)
		assert.Error(t, err)
		assert.Nil(t, wp)
	})
}
