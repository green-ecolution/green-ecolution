package vehicle

import (
	"context"
	"sort"
	"testing"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/stretchr/testify/assert"
)

func TestVehicleRepository_GetAll(t *testing.T) {
	t.Run("should return all verhicles ordered by water capacity and no limitation", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, len(allTestVehicles), len(got))
		assert.Equal(t, totalCount, int64(len(allTestVehicles)))

		sortedVehicles := sortVehicleByWaterCapacity(allTestVehicles)

		for i, vehicle := range got {
			assert.Equal(t, sortedVehicles[i].ID, vehicle.ID)
			assert.Equal(t, sortedVehicles[i].Description, vehicle.Description)
			assert.Equal(t, sortedVehicles[i].NumberPlate, vehicle.NumberPlate)
			assert.Equal(t, sortedVehicles[i].WaterCapacity, vehicle.WaterCapacity)
			assert.Equal(t, sortedVehicles[i].Type, vehicle.Type)
			assert.Equal(t, sortedVehicles[i].Status, vehicle.Status)
			assert.Equal(t, sortedVehicles[i].DrivingLicense, vehicle.DrivingLicense)
			assert.Equal(t, sortedVehicles[i].Height, vehicle.Height)
			assert.Equal(t, sortedVehicles[i].Width, vehicle.Width)
			assert.Equal(t, sortedVehicles[i].Length, vehicle.Length)
			assert.Equal(t, sortedVehicles[i].Weight, vehicle.Weight)
			assert.Equal(t, sortedVehicles[i].Model, vehicle.Model)
			assert.NotZero(t, vehicle.CreatedAt)
			assert.NotZero(t, vehicle.UpdatedAt)
		}
	})

	t.Run("should return all verhicles with provider", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		expectedVehicle := allTestVehicles[len(allTestVehicles)-1]

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{Provider: "test-provider"})

		// then
		assert.NoError(t, err)
		assert.Equal(t, totalCount, int64(1))
		assert.Equal(t, expectedVehicle.ID, got[0].ID)
		assert.Equal(t, expectedVehicle.Description, got[0].Description)
		assert.Equal(t, expectedVehicle.NumberPlate, got[0].NumberPlate)
		assert.Equal(t, expectedVehicle.WaterCapacity, got[0].WaterCapacity)
		assert.Equal(t, expectedVehicle.Type, got[0].Type)
		assert.Equal(t, expectedVehicle.Status, got[0].Status)
		assert.Equal(t, expectedVehicle.DrivingLicense, got[0].DrivingLicense)
		assert.Equal(t, expectedVehicle.Height, got[0].Height)
		assert.Equal(t, expectedVehicle.Width, got[0].Width)
		assert.Equal(t, expectedVehicle.Length, got[0].Length)
		assert.Equal(t, expectedVehicle.Weight, got[0].Weight)
		assert.Equal(t, expectedVehicle.Model, got[0].Model)
		assert.Equal(t, expectedVehicle.Provider, got[0].Provider)
		assert.Equal(t, expectedVehicle.AdditionalInfo, got[0].AdditionalInfo)
		assert.NotZero(t, got[0].CreatedAt)
		assert.NotZero(t, got[0].UpdatedAt)
	})

	t.Run("should return all verhicles ordered by water capacity limited by 1 and with an offset of 1", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(1))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.NoError(t, err)
		assert.Len(t, got, 1)
		assert.Equal(t, totalCount, int64(len(allTestVehicles)))

		sortedVehicles := sortVehicleByWaterCapacity(allTestVehicles)[0:1]

		for i, vehicle := range got {
			assert.Equal(t, sortedVehicles[i].ID, vehicle.ID)
		}
	})

	t.Run("should return error on invalid page value", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		ctx := context.WithValue(context.Background(), "page", int32(0))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error on invalid limit value", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(0))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
		assert.Equal(t, int64(0), totalCount)
	})

	t.Run("should return empty slice when db is empty", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, totalCount, int64(0))
	})
}

func TestVehicleRepository_GetAllByType(t *testing.T) {
	t.Run("should return all verhicles of type transporter without limitation", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAllByType(ctx, "", entities.VehicleTypeTransporter)

		// then
		assert.NoError(t, err)
		assert.Len(t, got, 2)
		assert.Equal(t, 2, len(got))
		assert.Equal(t, int64(2), totalCount)

		for _, vehicle := range got {
			assert.Equal(t, entities.VehicleTypeTransporter, vehicle.Type)
		}
	})

	t.Run("should return all verhicles of type trailer and no limitation", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAllByType(ctx, "", entities.VehicleTypeTrailer)

		// then
		assert.NoError(t, err)
		assert.Equal(t, 1, len(got))
		assert.Equal(t, totalCount, int64(1))

		for _, vehicle := range got {
			assert.Equal(t, entities.VehicleTypeTrailer, vehicle.Type)
		}
	})

	t.Run("should return all verhicles with provider and transporter type", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		expectedVehicle := allTestVehicles[len(allTestVehicles)-1]

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAllByType(ctx, "test-provider", entities.VehicleTypeTransporter)

		// then
		assert.NoError(t, err)
		assert.Equal(t, totalCount, int64(1))
		assert.Equal(t, expectedVehicle.ID, got[0].ID)
		assert.Equal(t, expectedVehicle.Provider, got[0].Provider)
		assert.Equal(t, expectedVehicle.AdditionalInfo, got[0].AdditionalInfo)
		assert.Equal(t, entities.VehicleTypeTransporter, got[0].Type)
	})

	t.Run("should return all verhicles of type trailer and with an limit of 1 and an offset of 2", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(1))

		// when
		got, totalCount, err := r.GetAllByType(ctx, "", entities.VehicleTypeTrailer)

		// then
		assert.NoError(t, err)
		assert.Equal(t, 0, len(got))
		assert.Equal(t, totalCount, int64(1))
	})

	t.Run("should return empty slice when db is empty", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAllByType(ctx, "", entities.VehicleTypeUnknown)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, totalCount, err := r.GetAllByType(ctx, "", entities.VehicleTypeUnknown)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, totalCount, int64(0))
	})
}

func TestVehicleRepository_GetAllWithWateringPlanCount(t *testing.T) {
	t.Run("should return all verhicles with the associated watering plan count", func(t *testing.T) {
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		exptectedVehicles := getVehicleWateringPlanCounts()

		got, err := r.GetAllWithWateringPlanCount(context.Background())

		assert.NoError(t, err)
		assert.Equal(t, len(exptectedVehicles), len(got))

		for i, entry := range got {
			assert.Equal(t, exptectedVehicles[i].NumberPlate, entry.NumberPlate)
			assert.Equal(t, exptectedVehicles[i].WateringPlanCount, entry.WateringPlanCount)
		}
	})

	t.Run("should return empty slice on empty db", func(t *testing.T) {
		suite.ResetDB(t)
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		got, err := r.GetAllWithWateringPlanCount(context.Background())

		assert.NoError(t, err)
		assert.Equal(t, 0, len(got))
		assert.Empty(t, got)
	})
}

func TestVehicleRepository_GetByID(t *testing.T) {
	t.Run("should return verhicle by id", func(t *testing.T) {
		// given
		ctx := context.Background()
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())
		shouldReturn := allTestVehicles[0]

		// when
		got, err := r.GetByID(ctx, 1)

		// then
		assert.NoError(t, err)
		assert.Equal(t, shouldReturn.ID, got.ID)
		assert.Equal(t, shouldReturn.NumberPlate, got.NumberPlate)
		assert.Equal(t, shouldReturn.Description, got.Description)
		assert.Equal(t, shouldReturn.WaterCapacity, got.WaterCapacity)
		assert.Equal(t, shouldReturn.Type, got.Type)
		assert.Equal(t, shouldReturn.Status, got.Status)
		assert.Equal(t, shouldReturn.DrivingLicense, got.DrivingLicense)
		assert.Equal(t, shouldReturn.Height, got.Height)
		assert.Equal(t, shouldReturn.Length, got.Length)
		assert.Equal(t, shouldReturn.Width, got.Width)
		assert.Equal(t, shouldReturn.Weight, got.Weight)
		assert.Equal(t, shouldReturn.Model, got.Model)
		assert.NotZero(t, got.CreatedAt)
		assert.NotZero(t, got.UpdatedAt)
	})

	t.Run("should return error when verhicle not found", func(t *testing.T) {
		// given
		ctx := context.Background()
		suite.ResetDB(t)
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		// when
		got, err := r.GetByID(ctx, 1)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when vehicle id is negative", func(t *testing.T) {
		// given
		ctx := context.Background()
		suite.ResetDB(t)
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		// when
		got, err := r.GetByID(ctx, -1)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when vehicle id is zero", func(t *testing.T) {
		// given
		ctx := context.Background()
		suite.ResetDB(t)
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		// when
		got, err := r.GetByID(ctx, 0)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.GetByID(ctx, 1)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestVehicleRepository_GetByPlate(t *testing.T) {
	tests := []struct {
		name string
		want *entities.Vehicle
		args string
	}{
		{
			name: "should return region by plate 'B-1234'",
			want: allTestVehicles[0],
			args: "B-1234",
		},
		{
			name: "should return region by name 'B-5678'",
			want: allTestVehicles[1],
			args: "B-5678",
		},
	}

	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// given
			ctx := context.Background()
			r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

			// when
			got, err := r.GetByPlate(ctx, tt.args)

			// then
			assert.NoError(t, err)
			assert.Equal(t, tt.want.ID, got.ID)
			assert.Equal(t, tt.want.NumberPlate, got.NumberPlate)
			assert.Equal(t, tt.want.Description, got.Description)
			assert.Equal(t, tt.want.WaterCapacity, got.WaterCapacity)
			assert.Equal(t, tt.want.Type, got.Type)
			assert.Equal(t, tt.want.Status, got.Status)
			assert.Equal(t, tt.want.DrivingLicense, got.DrivingLicense)
			assert.Equal(t, tt.want.Height, got.Height)
			assert.Equal(t, tt.want.Length, got.Length)
			assert.Equal(t, tt.want.Width, got.Width)
			assert.Equal(t, tt.want.Weight, got.Weight)
			assert.Equal(t, tt.want.Model, got.Model)
			assert.NotZero(t, got.CreatedAt)
			assert.NotZero(t, got.UpdatedAt)
		})
	}

	t.Run("should return error when vehicle not found", func(t *testing.T) {
		// given
		ctx := context.Background()
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		// when
		got, err := r.GetByPlate(ctx, "Non-existing")

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when vehicle plate is empty", func(t *testing.T) {
		// given
		ctx := context.Background()
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		// when
		got, err := r.GetByPlate(ctx, "")

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.GetByPlate(ctx, "B-1234")

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

var allTestVehicles = []*entities.Vehicle{
	{
		ID:             1,
		NumberPlate:    "B-1234",
		Description:    "Test vehicle 1",
		WaterCapacity:  100.0,
		Type:           entities.VehicleTypeTrailer,
		Status:         entities.VehicleStatusActive,
		Model:          "1615/17 - Conrad - MAN TGE 3.180",
		DrivingLicense: entities.DrivingLicenseBE,
		Height:         1.5,
		Length:         2.0,
		Width:          2.0,
		Weight:         3.3,
	},
	{
		ID:             2,
		NumberPlate:    "B-5678",
		Description:    "Test vehicle 2",
		WaterCapacity:  150.0,
		Type:           entities.VehicleTypeTransporter,
		Status:         entities.VehicleStatusUnknown,
		Model:          "Actros L Mercedes Benz",
		DrivingLicense: entities.DrivingLicenseC,
		Height:         2.1,
		Length:         5.0,
		Width:          2.4,
		Weight:         3.7,
	},
	{
		ID:             3,
		NumberPlate:    "B-1001",
		Description:    "Test vehicle 3",
		WaterCapacity:  150.0,
		Type:           entities.VehicleTypeTransporter,
		Status:         entities.VehicleStatusUnknown,
		Model:          "Actros L Mercedes Benz",
		DrivingLicense: entities.DrivingLicenseC,
		Height:         2.1,
		Length:         5.0,
		Width:          2.4,
		Weight:         3.7,
		Provider:       "test-provider",
		AdditionalInfo: map[string]interface{}{
			"foo": "bar",
		},
	},
}

func sortVehicleByWaterCapacity(data []*entities.Vehicle) []*entities.Vehicle {
	sorted := make([]*entities.Vehicle, len(data))
	copy(sorted, data)

	sort.Slice(sorted, func(i, j int) bool {
		return sorted[i].WaterCapacity > sorted[j].WaterCapacity
	})

	return sorted
}

func getVehicleWateringPlanCounts() []*entities.VehicleEvaluation {
	vehicleCountMap := make(map[string]int)

	for _, plan := range allTestWateringPlans {
		if plan.Transporter != nil && plan.Transporter.NumberPlate != "" {
			vehicleCountMap[plan.Transporter.NumberPlate]++
		}
		if plan.Trailer != nil && plan.Trailer.NumberPlate != "" {
			vehicleCountMap[plan.Trailer.NumberPlate]++
		}
	}

	var vehicleCounts []*entities.VehicleEvaluation
	for plate, count := range vehicleCountMap {
		vehicleCounts = append(vehicleCounts, &entities.VehicleEvaluation{
			NumberPlate:       plate,
			WateringPlanCount: int64(count),
		})
	}

	sort.Slice(vehicleCounts, func(i, j int) bool {
		return vehicleCounts[i].WateringPlanCount > vehicleCounts[j].WateringPlanCount
	})

	return vehicleCounts
}

type testWateringPlan struct {
	ID          int32
	Date        time.Time
	Transporter *entities.Vehicle
	Trailer     *entities.Vehicle
}

var allTestWateringPlans = []*testWateringPlan{
	{
		ID:          1,
		Date:        time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
		Transporter: allTestVehicles[1],
		Trailer:     allTestVehicles[0],
	},
	{
		ID:          2,
		Date:        time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
		Transporter: allTestVehicles[1],
		Trailer:     nil,
	},
	{
		ID:          3,
		Date:        time.Date(2024, 6, 12, 0, 0, 0, 0, time.UTC),
		Transporter: allTestVehicles[1],
		Trailer:     nil,
	},
	{
		ID:          4,
		Date:        time.Date(2024, 6, 10, 0, 0, 0, 0, time.UTC),
		Transporter: allTestVehicles[1],
		Trailer:     nil,
	},
	{
		ID:          5,
		Date:        time.Date(2024, 6, 4, 0, 0, 0, 0, time.UTC),
		Transporter: allTestVehicles[1],
	},
}
