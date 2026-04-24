package vehicle

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
)

func TestVehicleService_GetAll(t *testing.T) {
	ctx := context.Background()

	t.Run("should return all vehicles with no provider and no vehicle type when successful", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		expectedVehicles := getTestVehicles()
		vehicleRepo.EXPECT().GetAll(ctx, shared.Query{}).Return(expectedVehicles, int64(len(expectedVehicles)), nil)

		// when
		vehicles, totalCount, err := svc.GetAll(ctx, vehicle.VehicleQuery{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedVehicles, vehicles)
		assert.Equal(t, totalCount, int64(len(expectedVehicles)))
	})

	t.Run("should return all vehicles when successful with provider", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		expectedVehicles := getTestVehicles()

		vehicleRepo.EXPECT().GetAll(ctx, shared.Query{Provider: "test-provider"}).Return(expectedVehicles, int64(len(expectedVehicles)), nil)

		// when
		vehicles, totalCount, err := svc.GetAll(ctx, vehicle.VehicleQuery{Query: shared.Query{Provider: "test-provider"}})

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedVehicles, vehicles)
		assert.Equal(t, totalCount, int64(len(expectedVehicles)))
	})

	t.Run("should return all vehicles when successful with vehicle type", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		expectedVehicles := getTestVehicles()
		vehicleRepo.EXPECT().GetAllByType(ctx, "", vehicle.VehicleTypeTrailer).Return(expectedVehicles, int64(len(expectedVehicles)), nil)

		// when
		vehicles, totalCount, err := svc.GetAll(ctx, vehicle.VehicleQuery{Type: "trailer"})

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedVehicles, vehicles)
		assert.Equal(t, totalCount, int64(len(expectedVehicles)))
	})

	t.Run("should return all vehicles when successful with provider and vehicle type", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		expectedVehicles := getTestVehicles()
		vehicleRepo.EXPECT().GetAllByType(ctx, "test-provider", vehicle.VehicleTypeTrailer).Return(expectedVehicles, int64(len(expectedVehicles)), nil)

		// when
		vehicles, totalCount, err := svc.GetAll(ctx, vehicle.VehicleQuery{Query: shared.Query{Provider: "test-provider"}, Type: "trailer"})

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedVehicles, vehicles)
		assert.Equal(t, totalCount, int64(len(expectedVehicles)))
	})

	t.Run("should return empty slice when no vehicles are found", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		vehicleRepo.EXPECT().GetAll(ctx, shared.Query{}).Return([]*vehicle.Vehicle{}, int64(0), nil)

		// when
		vehicles, totalCount, err := svc.GetAll(ctx, vehicle.VehicleQuery{})

		// then
		assert.NoError(t, err)
		assert.Empty(t, vehicles)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error when GetAll fails", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		expectedErr := errors.New("GetAll failed")
		vehicleRepo.EXPECT().GetAll(ctx, shared.Query{}).Return(nil, int64(0), expectedErr)

		// when
		vehicles, totalCount, err := svc.GetAll(ctx, vehicle.VehicleQuery{})

		// then
		assert.Error(t, err)
		assert.Nil(t, vehicles)
		assert.Equal(t, totalCount, int64(0))
		// assert.EqualError(t, err, "500: GetAll failed")
	})
}

func TestVehicleService_GetByID(t *testing.T) {
	ctx := context.Background()

	t.Run("should return vehicle when found", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		id := int32(1)
		expectedVehicle := getTestVehicles()[0]
		vehicleRepo.EXPECT().GetByID(ctx, id).Return(expectedVehicle, nil)

		// when
		vehicle, err := svc.GetByID(ctx, id)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedVehicle, vehicle)
	})

	t.Run("should return error if vehicle not found", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		id := int32(1)
		expectedErr := shared.ErrEntityNotFound("not found")
		vehicleRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedErr)

		// when
		vehicle, err := svc.GetByID(ctx, id)

		// then
		assert.Error(t, err)
		assert.Nil(t, vehicle)
		// assert.EqualError(t, err, "404: vehicle not found")
	})
}

func TestVehicleService_GetByPlate(t *testing.T) {
	ctx := context.Background()

	t.Run("should return vehicle when found", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		plate := "FL TBZ 1234"
		expectedVehicle := getTestVehicles()[0]
		vehicleRepo.EXPECT().GetByPlate(ctx, plate).Return(expectedVehicle, nil)

		// when
		vehicle, err := svc.GetByPlate(ctx, plate)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedVehicle, vehicle)
	})

	t.Run("should return error if vehicle not found", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		plate := "FL TBZ 1234"
		expectedErr := shared.ErrEntityNotFound("not found")
		vehicleRepo.EXPECT().GetByPlate(ctx, plate).Return(nil, expectedErr)

		// when
		vehicle, err := svc.GetByPlate(ctx, plate)

		// then
		assert.Error(t, err)
		assert.Nil(t, vehicle)
		// assert.EqualError(t, err, "404: vehicle not found")
	})
}

func TestVehicleService_Create(t *testing.T) {
	ctx := context.Background()
	input := &vehicle.VehicleCreate{
		NumberPlate:    "FL TBZ 123",
		Description:    "Test description",
		Status:         vehicle.VehicleStatusActive,
		Type:           vehicle.VehicleTypeTrailer,
		WaterCapacity:  shared.MustNewWaterCapacity(2000.5),
		Model:          "Actros L Mercedes Benz",
		DrivingLicense: vehicle.DrivingLicenseBE,
		Height:         2.1,
		Length:         5.0,
		Width:          2.4,
		Weight:         3.2,
	}

	t.Run("should successfully create a new vehicle", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		expectedVehicle := getTestVehicles()[0]

		vehicleRepo.EXPECT().GetByPlate(
			ctx,
			input.NumberPlate,
		).Return(nil, nil)

		vehicleRepo.EXPECT().Create(
			ctx,
			mock.Anything,
		).Return(expectedVehicle, nil)

		// when
		result, err := svc.Create(ctx, input)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedVehicle, result)
	})

	t.Run("should return an error when creating vehicle fails", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		expectedErr := errors.New("Failed to create vehicle")

		vehicleRepo.EXPECT().GetByPlate(
			ctx,
			input.NumberPlate,
		).Return(nil, nil)

		vehicleRepo.EXPECT().Create(
			ctx,
			mock.Anything,
		).Return(nil, expectedErr)

		// when
		result, err := svc.Create(ctx, input)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.EqualError(t, err, "500: Failed to create vehicle")
	})

	t.Run("should return an error when creating vehicle fails due to dupliacte number plate", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		vehicleRepo.EXPECT().GetByPlate(
			ctx,
			input.NumberPlate,
		).Return(getTestVehicles()[0], nil)

		// when
		result, err := svc.Create(ctx, input)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.EqualError(t, err, "400: number plate is already taken")
	})

	t.Run("should return an error when creating vehicle fails due to error in GetByPlate", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		expectedErr := errors.New("failed to get vehicle")

		vehicleRepo.EXPECT().GetByPlate(
			ctx,
			input.NumberPlate,
		).Return(nil, expectedErr)

		// when
		result, err := svc.Create(ctx, input)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.EqualError(t, err, "500: failed to get vehicle")
	})

}

func TestVehicleService_Update(t *testing.T) {
	ctx := context.Background()
	vehicleID := int32(1)
	input := &vehicle.VehicleUpdate{
		NumberPlate:    "FL TBZ 123",
		Description:    "Test description",
		Status:         vehicle.VehicleStatusActive,
		Type:           vehicle.VehicleTypeTrailer,
		WaterCapacity:  shared.MustNewWaterCapacity(2000.5),
		Model:          "Actros L Mercedes Benz",
		DrivingLicense: vehicle.DrivingLicenseBE,
		Height:         2.1,
		Length:         5.0,
		Width:          2.4,
		Weight:         3.2,
	}

	t.Run("should successfully update a vehicle", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		expectedVehicle := getTestVehicles()[0]

		vehicleRepo.EXPECT().GetByID(
			ctx,
			vehicleID,
		).Return(expectedVehicle, nil)

		vehicleRepo.EXPECT().Update(
			ctx,
			vehicleID,
			mock.Anything,
		).Return(nil)

		vehicleRepo.EXPECT().GetByID(
			ctx,
			vehicleID,
		).Return(expectedVehicle, nil)

		// when
		result, err := svc.Update(ctx, vehicleID, input)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedVehicle, result)
	})

	t.Run("should return an error when vehicle is not found", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		vehicleRepo.EXPECT().GetByID(
			ctx,
			vehicleID,
		).Return(nil, shared.ErrEntityNotFound("not found"))

		// when
		result, err := svc.Update(ctx, vehicleID, input)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.EqualError(t, err, "404: vehicle not found")
	})

	t.Run("should return an error when the update fails", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		expectedErr := errors.New("failed to update vehicle")
		expectedVehicle := getTestVehicles()[0]

		vehicleRepo.EXPECT().GetByID(
			ctx,
			vehicleID,
		).Return(expectedVehicle, nil)

		vehicleRepo.EXPECT().Update(
			ctx,
			vehicleID,
			mock.Anything,
		).Return(expectedErr)

		// when
		result, err := svc.Update(ctx, vehicleID, input)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.EqualError(t, err, "500: failed to update vehicle")
	})

	t.Run("should return an error when updating vehicle fails due to dupliacte number plate", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		input.NumberPlate = "1234"

		vehicleRepo.EXPECT().GetByID(
			ctx,
			vehicleID,
		).Return(getTestVehicles()[0], nil)

		vehicleRepo.EXPECT().GetByPlate(
			ctx,
			input.NumberPlate,
		).Return(getTestVehicles()[1], nil)

		// when
		result, err := svc.Update(ctx, vehicleID, input)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.EqualError(t, err, "400: number plate is already taken")
	})

	t.Run("should return an error when updating vehicle fails due to error in GetByPlate", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		expectedErr := errors.New("failed to get vehicle")

		vehicleRepo.EXPECT().GetByID(
			ctx,
			vehicleID,
		).Return(getTestVehicles()[0], nil)

		vehicleRepo.EXPECT().GetByPlate(
			ctx,
			input.NumberPlate,
		).Return(nil, expectedErr)

		// when
		result, err := svc.Update(ctx, vehicleID, input)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		// assert.EqualError(t, err, "500: failed to get vehicle")
	})

}

func TestVehicleService_Delete(t *testing.T) {
	ctx := context.Background()

	t.Run("should successfully delete a vehicle", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		id := int32(1)

		vehicleRepo.EXPECT().GetByID(ctx, id).Return(getTestVehicles()[0], nil)
		vehicleRepo.EXPECT().Delete(ctx, id).Return(nil)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.NoError(t, err)
	})

	t.Run("should return error if vehicle not found", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		id := int32(1)
		expectedErr := shared.ErrEntityNotFound("not found")
		vehicleRepo.EXPECT().GetByID(ctx, id).Return(nil, expectedErr)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "404: vehicle not found")
	})

	t.Run("should return error if deleting vehicle fails", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		id := int32(4)
		expectedErr := errors.New("failed to delete")

		vehicleRepo.EXPECT().GetByID(ctx, id).Return(getTestVehicles()[0], nil)
		vehicleRepo.EXPECT().Delete(ctx, id).Return(expectedErr)

		// when
		err := svc.Delete(ctx, id)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to delete")
	})
}

func TestReady(t *testing.T) {
	t.Run("should return true if the service is ready", func(t *testing.T) {
		vehicleRepo := storageMock.NewMockVehicleRepository(t)
		svc := NewVehicleService(vehicleRepo)

		// when
		ready := svc.Ready()

		// then
		assert.True(t, ready)
	})

	t.Run("should return false if the service is not ready", func(t *testing.T) {
		svc := NewVehicleService(nil)

		// when
		ready := svc.Ready()

		// then
		assert.False(t, ready)
	})
}

func getTestVehicles() []*vehicle.Vehicle {
	now := time.Now()

	return []*vehicle.Vehicle{
		{
			ID:             1,
			CreatedAt:      now,
			UpdatedAt:      now,
			NumberPlate:    "FL TBZ 123",
			Description:    "Test description",
			Status:         vehicle.VehicleStatusActive,
			Type:           vehicle.VehicleTypeTrailer,
			WaterCapacity:  shared.MustNewWaterCapacity(2000.5),
			Model:          "1615/17 - Conrad - MAN TGE 3.180",
			DrivingLicense: vehicle.DrivingLicenseBE,
			Height:         1.5,
			Length:         2.0,
			Width:          2.0,
			Weight:         3.2,
		},
		{
			ID:             2,
			CreatedAt:      now,
			UpdatedAt:      now,
			NumberPlate:    "FL TBZ 3456",
			Description:    "Test description",
			Status:         vehicle.VehicleStatusNotAvailable,
			Type:           vehicle.VehicleTypeTransporter,
			WaterCapacity:  shared.MustNewWaterCapacity(1000.5),
			Model:          "Actros L Mercedes Benz",
			DrivingLicense: vehicle.DrivingLicenseC,
			Height:         2.1,
			Length:         5.0,
			Width:          2.4,
			Weight:         3.5,
		},
	}
}
