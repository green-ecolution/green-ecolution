package mapper_test

import (
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/mapper"
)

func TestVehicleMapper_FromSql(t *testing.T) {
	verhicleMapper := &mapper.InternalVehicleRepoMapperImpl{}

	t.Run("should convert from sql to entity", func(t *testing.T) {
		// given
		src := allTestVehicles[0]

		// when
		got, err := verhicleMapper.FromSql(src)

		// then
		assert.NotNil(t, got)
		assert.NoError(t, err)
		assert.Equal(t, src.ID, got.ID)
		assert.Equal(t, src.CreatedAt, got.CreatedAt)
		assert.Equal(t, src.UpdatedAt, got.UpdatedAt)
		assert.Equal(t, src.NumberPlate, got.NumberPlate)
		assert.Equal(t, src.Description, got.Description)
		assert.Equal(t, src.WaterCapacity, got.WaterCapacity.Liters())
		assert.Equal(t, src.Model, got.Model)
		assert.Equal(t, src.Length, got.Length)
		assert.Equal(t, src.Height, got.Height)
		assert.Equal(t, src.Width, got.Width)
		assert.Equal(t, src.Length, got.Length)
		assert.Equal(t, src.Type, sqlc.VehicleType(got.Type))
		assert.Equal(t, src.Status, sqlc.VehicleStatus(got.Status))
		assert.Equal(t, src.DrivingLicense, sqlc.DrivingLicense(got.DrivingLicense))
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src *sqlc.Vehicle = nil

		// when
		got, err := verhicleMapper.FromSql(src)

		// then
		assert.Nil(t, got)
		assert.NoError(t, err)
	})
}

func TestVehicleMapper_FromSqlList(t *testing.T) {
	verhicleMapper := &mapper.InternalVehicleRepoMapperImpl{}

	t.Run("should convert from sql slice to entity slice", func(t *testing.T) {
		// given
		src := allTestVehicles

		// when
		got, err := verhicleMapper.FromSqlList(src)

		// then
		assert.NotNil(t, got)
		assert.NoError(t, err)
		assert.Len(t, got, 2)

		for i, src := range src {
			assert.NotNil(t, got)
			assert.Equal(t, src.ID, got[i].ID)
			assert.Equal(t, src.CreatedAt, got[i].CreatedAt)
			assert.Equal(t, src.UpdatedAt, got[i].UpdatedAt)
			assert.Equal(t, src.NumberPlate, got[i].NumberPlate)
			assert.Equal(t, src.Description, got[i].Description)
			assert.Equal(t, src.WaterCapacity, got[i].WaterCapacity.Liters())
			assert.Equal(t, src.Model, got[i].Model)
			assert.Equal(t, src.Width, got[i].Width)
			assert.Equal(t, src.Length, got[i].Length)
			assert.Equal(t, src.Height, got[i].Height)
			assert.Equal(t, src.Type, sqlc.VehicleType(got[i].Type))
			assert.Equal(t, src.Status, sqlc.VehicleStatus(got[i].Status))
			assert.Equal(t, src.DrivingLicense, sqlc.DrivingLicense(got[i].DrivingLicense))
		}
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src []*sqlc.Vehicle = nil

		// when
		got, err := verhicleMapper.FromSqlList(src)

		// then
		assert.Nil(t, got)
		assert.NoError(t, err)
	})
}

var allTestVehicles = []*sqlc.Vehicle{
	{
		ID:             1,
		CreatedAt:      time.Now(),
		UpdatedAt:      time.Now(),
		NumberPlate:    "FL TZ 1234",
		Description:    "This is a big car",
		WaterCapacity:  2000.10,
		Type:           sqlc.VehicleTypeTransporter,
		Status:         sqlc.VehicleStatusNotavailable,
		Model:          "1615/17 - Conrad - MAN TGE 3.180",
		DrivingLicense: sqlc.DrivingLicenseBE,
		Height:         1.5,
		Length:         2.0,
		Width:          2.0,
	},
	{
		ID:             2,
		CreatedAt:      time.Now(),
		UpdatedAt:      time.Now(),
		NumberPlate:    "FL TZ 1235",
		Description:    "This is a small car",
		WaterCapacity:  1000,
		Type:           sqlc.VehicleTypeTransporter,
		Status:         sqlc.VehicleStatusNotavailable,
		Model:          "Actros L Mercedes Benz",
		DrivingLicense: sqlc.DrivingLicenseC,
		Height:         2.1,
		Length:         5.0,
		Width:          2.4,
	},
}

func TestMapVehicleStatus(t *testing.T) {
	tests := []struct {
		input    sqlc.VehicleStatus
		expected shared.VehicleStatus
	}{
		{input: sqlc.VehicleStatusActive, expected: shared.VehicleStatusActive},
		{input: sqlc.VehicleStatusAvailable, expected: shared.VehicleStatusAvailable},
		{input: sqlc.VehicleStatusNotavailable, expected: shared.VehicleStatusNotAvailable},
		{input: sqlc.VehicleStatusUnknown, expected: shared.VehicleStatusUnknown},
	}

	for _, test := range tests {
		t.Run(fmt.Sprintf("should return %v for input %v", test.expected, test.input), func(t *testing.T) {
			result := mapper.MapVehicleStatus(test.input)
			assert.Equal(t, test.expected, result)
		})
	}
}

func TestMapVehicleType(t *testing.T) {
	tests := []struct {
		input    sqlc.VehicleType
		expected shared.VehicleType
	}{
		{input: sqlc.VehicleTypeTrailer, expected: shared.VehicleTypeTrailer},
		{input: sqlc.VehicleTypeTransporter, expected: shared.VehicleTypeTransporter},
		{input: sqlc.VehicleTypeUnknown, expected: shared.VehicleTypeUnknown},
	}

	for _, test := range tests {
		t.Run(fmt.Sprintf("should return %v for input %v", test.expected, test.input), func(t *testing.T) {
			result := mapper.MapVehicleType(test.input)
			assert.Equal(t, test.expected, result)
		})
	}
}

func TestMapDrivingLicense(t *testing.T) {
	tests := []struct {
		input    sqlc.DrivingLicense
		expected shared.DrivingLicense
	}{
		{input: sqlc.DrivingLicenseB, expected: shared.DrivingLicenseB},
		{input: sqlc.DrivingLicenseBE, expected: shared.DrivingLicenseBE},
		{input: sqlc.DrivingLicenseC, expected: shared.DrivingLicenseC},
		{input: sqlc.DrivingLicenseCE, expected: shared.DrivingLicenseCE},
	}

	for _, test := range tests {
		t.Run(fmt.Sprintf("should return %v for input %v", test.expected, test.input), func(t *testing.T) {
			result := mapper.MapDrivingLicense(test.input)
			assert.Equal(t, test.expected, result)
		})
	}
}
