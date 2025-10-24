package wateringplan

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
)

func TestWateringPlanRepository_GetAll(t *testing.T) {
	t.Run("should return all watering plans without limitation", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")
		r := NewWateringPlanRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Len(t, got, len(allTestWateringPlans))
		assert.Equal(t, totalCount, int64(len(allTestWateringPlans)))

		for i, wp := range got {
			assert.Equal(t, allTestWateringPlans[i].ID, wp.ID)
			assert.Equal(t, allTestWateringPlans[i].Date, wp.Date)
			assert.Equal(t, allTestWateringPlans[i].Description, wp.Description)
			assert.Equal(t, allTestWateringPlans[i].Status, wp.Status)
			assert.Equal(t, allTestWateringPlans[i].Distance, wp.Distance)
			assert.Equal(t, *allTestWateringPlans[i].TotalWaterRequired, *wp.TotalWaterRequired)
			assert.Equal(t, allTestWateringPlans[i].CancellationNote, wp.CancellationNote)

			// assert transporter
			assert.Equal(t, allTestWateringPlans[i].Transporter.ID, wp.Transporter.ID)

			// assert trailer
			if allTestWateringPlans[i].Trailer == nil {
				assert.Nil(t, wp.Trailer)
				assert.NoError(t, err)
			} else {
				assert.NotNil(t, wp.Trailer)
				assert.Equal(t, allTestWateringPlans[i].Trailer.ID, wp.Trailer.ID)
			}

			// assert treecluster
			assert.Len(t, allTestWateringPlans[i].TreeClusters, len(wp.TreeClusters))
			for j, tc := range wp.TreeClusters {
				assert.Equal(t, allTestWateringPlans[i].TreeClusters[j].ID, tc.ID)
				assert.Equal(t, allTestWateringPlans[i].TreeClusters[j].Name, tc.Name)
			}

			// assert user
			assert.Len(t, allTestWateringPlans[i].UserIDs, len(wp.UserIDs))
			for j, userID := range wp.UserIDs {
				assert.Equal(t, allTestWateringPlans[i].UserIDs[j], userID)
			}

			// assert evaluation
			if allTestWateringPlans[i].Evaluation == nil {
				assert.Len(t, allTestWateringPlans[i].Evaluation, 0)
				// check if evaluation is empty if the status is not finished
				assert.NotEqual(t, entities.WateringPlanStatusFinished, wp.Status)
			} else {
				assert.Equal(t, len(allTestWateringPlans[i].Evaluation), len(wp.Evaluation))
				assert.Equal(t, entities.WateringPlanStatusFinished, wp.Status)
				for j, value := range wp.Evaluation {
					assert.Equal(t, allTestWateringPlans[i].Evaluation[j].WateringPlanID, value.WateringPlanID)
					assert.Equal(t, allTestWateringPlans[i].Evaluation[j].TreeClusterID, value.TreeClusterID)
					assert.Equal(t, *allTestWateringPlans[i].Evaluation[j].ConsumedWater, *value.ConsumedWater)
				}
			}
		}
	})

	t.Run("should return all watering plans with provider", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")
		r := NewWateringPlanRepository(suite.Store, mappers)

		expectedPlan := allTestWateringPlans[1]

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{Provider: "test-provider"})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Equal(t, totalCount, int64(1))
		assert.Equal(t, expectedPlan.ID, got[0].ID)
		assert.Equal(t, expectedPlan.Date, got[0].Date)
		assert.Equal(t, expectedPlan.Description, got[0].Description)
		assert.Equal(t, expectedPlan.Status, got[0].Status)
		assert.Equal(t, expectedPlan.Distance, got[0].Distance)
		assert.Equal(t, *expectedPlan.TotalWaterRequired, *got[0].TotalWaterRequired)
		assert.Equal(t, expectedPlan.CancellationNote, got[0].CancellationNote)
	})

	t.Run("should return all watering plans limited by 2 and with an offset of 2", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")
		r := NewWateringPlanRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})
		wateringPlans := allTestWateringPlans[2:4]

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Len(t, got, 2)
		assert.Equal(t, totalCount, int64(len(allTestWateringPlans)))

		for i, wp := range got {
			assert.Equal(t, wateringPlans[i].ID, wp.ID)
		}
	})

	t.Run("should return error on invalid page value", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")
		r := NewWateringPlanRepository(suite.Store, mappers)

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
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")
		r := NewWateringPlanRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(0))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return empty slice when db is empty", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewWateringPlanRepository(suite.Store, mappers)

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
		r := NewWateringPlanRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		_, _, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.Error(t, err)
	})
}

func TestWateringPlanRepository_GetCount(t *testing.T) {
	t.Run("should return count of all watering plans in db", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")
		r := NewWateringPlanRepository(suite.Store, mappers)
		// when
		totalCount, err := r.GetCount(context.Background(), entities.Query{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, int64(len(allTestWateringPlans)), totalCount)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		totalCount, err := r.GetCount(ctx, entities.Query{})

		// then
		assert.Error(t, err)
		assert.Equal(t, int64(0), totalCount)
	})
}

func TestWateringPlanRepository_GetTotalConsumedWater(t *testing.T) {
	t.Run("should return count of all watering plans in db", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")
		r := NewWateringPlanRepository(suite.Store, mappers)

		expectedTotalCount := calculateCountTotalWater()

		// when
		totalConsumedWater, err := r.GetTotalConsumedWater(context.Background())

		// then
		assert.NoError(t, err)
		assert.Equal(t, int64(expectedTotalCount), totalConsumedWater)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		totalConsumedWater, err := r.GetTotalConsumedWater(ctx)

		// then
		assert.Error(t, err)
		assert.Equal(t, int64(0), totalConsumedWater)
	})
}

func TestWateringPlanRepository_GetByID(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")

	t.Run("should return watering plan by id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, allTestWateringPlans[0].ID, got.ID)
		assert.Equal(t, allTestWateringPlans[0].Date, got.Date)
		assert.Equal(t, allTestWateringPlans[0].Description, got.Description)
		assert.Equal(t, allTestWateringPlans[0].Status, got.Status)
		assert.Equal(t, allTestWateringPlans[0].Distance, got.Distance)
		assert.Equal(t, *allTestWateringPlans[0].TotalWaterRequired, *got.TotalWaterRequired)
		assert.Equal(t, allTestWateringPlans[0].CancellationNote, got.CancellationNote)

		// assert transporter
		assert.Equal(t, allTestWateringPlans[0].Transporter.ID, got.Transporter.ID)
		assert.Equal(t, allTestWateringPlans[0].Transporter.Type, got.Transporter.Type)

		// assert trailer
		assert.Equal(t, allTestWateringPlans[0].Trailer.ID, got.Trailer.ID)
		assert.Equal(t, allTestWateringPlans[0].Trailer.Type, got.Trailer.Type)

		// assert treecluster
		assert.Len(t, got.TreeClusters, 2)
		for i, tc := range got.TreeClusters {
			assert.Equal(t, allTestWateringPlans[0].TreeClusters[i].ID, tc.ID)
			assert.Equal(t, allTestWateringPlans[0].TreeClusters[i].Name, tc.Name)
		}

		// assert user
		assert.Len(t, allTestWateringPlans[0].UserIDs, len(got.UserIDs))
		for j, userID := range got.UserIDs {
			assert.Equal(t, allTestWateringPlans[0].UserIDs[j], userID)
		}

		// assert evaluation
		assert.Len(t, allTestWateringPlans[0].Evaluation, 0)
	})

	t.Run("should return watering plan by id without trailer", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetByID(context.Background(), 2)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, allTestWateringPlans[1].ID, got.ID)
		assert.Equal(t, allTestWateringPlans[1].Date, got.Date)
		assert.Equal(t, allTestWateringPlans[1].Description, got.Description)
		assert.Equal(t, allTestWateringPlans[1].Status, got.Status)
		assert.Equal(t, allTestWateringPlans[1].Distance, got.Distance)
		assert.Equal(t, *allTestWateringPlans[1].TotalWaterRequired, *got.TotalWaterRequired)
		assert.Equal(t, allTestWateringPlans[1].CancellationNote, got.CancellationNote)

		// assert transporter
		assert.Equal(t, allTestWateringPlans[1].Transporter.ID, got.Transporter.ID)
		assert.Equal(t, allTestWateringPlans[1].Transporter.Type, got.Transporter.Type)

		// assert nil trailer
		assert.Nil(t, got.Trailer)

		// assert treecluster
		assert.Len(t, got.TreeClusters, 1)
		for i, tc := range got.TreeClusters {
			assert.Equal(t, allTestWateringPlans[1].TreeClusters[i].ID, tc.ID)
			assert.Equal(t, allTestWateringPlans[1].TreeClusters[i].Name, tc.Name)
		}

		// assert user
		assert.Len(t, allTestWateringPlans[1].UserIDs, len(got.UserIDs))
		for j, userID := range got.UserIDs {
			assert.Equal(t, allTestWateringPlans[1].UserIDs[j], userID)
		}
	})

	t.Run("should return watering plan by id with evaluation", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetByID(context.Background(), 3)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, entities.WateringPlanStatusFinished, got.Status)

		// assert evaluation
		assert.Equal(t, len(allTestWateringPlans[2].Evaluation), len(got.Evaluation))
		assert.Equal(t, entities.WateringPlanStatusFinished, got.Status)
		for j, value := range got.Evaluation {
			assert.Equal(t, allTestWateringPlans[2].Evaluation[j].WateringPlanID, value.WateringPlanID)
			assert.Equal(t, allTestWateringPlans[2].Evaluation[j].TreeClusterID, value.TreeClusterID)
			assert.Equal(t, *allTestWateringPlans[2].Evaluation[j].ConsumedWater, *value.ConsumedWater)
		}
	})

	t.Run("should return error when watering plan with non-existing id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetByID(context.Background(), 99)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when watering plan with negative id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetByID(context.Background(), -1)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when watering plan with zero id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetByID(context.Background(), 0)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.GetByID(ctx, 1)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestWateringPlanRepository_GetLinkedVehicleByIDAndType(t *testing.T) {
	ctx := context.Background()
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")

	t.Run("should return vehicle with type transporter by watering plan id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		shouldReturn := allTestVehicles[1]

		// when
		got, err := r.GetLinkedVehicleByIDAndType(ctx, int32(1), entities.VehicleTypeTransporter)

		// then
		assert.NoError(t, err)
		assert.Equal(t, shouldReturn.ID, got.ID)
		assert.Equal(t, shouldReturn.NumberPlate, got.NumberPlate)
		assert.Equal(t, shouldReturn.Description, got.Description)
		assert.Equal(t, shouldReturn.WaterCapacity, got.WaterCapacity)
		assert.Equal(t, entities.VehicleTypeTransporter, got.Type)
		assert.Equal(t, shouldReturn.Status, got.Status)
		assert.NotZero(t, got.CreatedAt)
		assert.NotZero(t, got.UpdatedAt)
	})

	t.Run("should return vehicle with type trailer by watering plan id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		shouldReturn := allTestVehicles[0]

		// when
		got, err := r.GetLinkedVehicleByIDAndType(ctx, int32(1), entities.VehicleTypeTrailer)

		// then
		assert.NoError(t, err)
		assert.Equal(t, shouldReturn.ID, got.ID)
		assert.Equal(t, shouldReturn.NumberPlate, got.NumberPlate)
		assert.Equal(t, shouldReturn.Description, got.Description)
		assert.Equal(t, shouldReturn.WaterCapacity, got.WaterCapacity)
		assert.Equal(t, entities.VehicleTypeTrailer, got.Type)
		assert.Equal(t, shouldReturn.Status, got.Status)
		assert.NotZero(t, got.CreatedAt)
		assert.NotZero(t, got.UpdatedAt)
	})

	t.Run("should return error when watering plan not found", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedVehicleByIDAndType(ctx, int32(99), entities.VehicleTypeTrailer)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when vehicle with type trailer is not found", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedVehicleByIDAndType(ctx, int32(2), entities.VehicleTypeTrailer)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when watering plan id is negative", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedVehicleByIDAndType(ctx, int32(-1), entities.VehicleTypeTransporter)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when watering plan id is zero", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedVehicleByIDAndType(ctx, int32(0), entities.VehicleTypeTransporter)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when vehicle type is not trailer or transporter", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedVehicleByIDAndType(ctx, int32(1), entities.VehicleTypeUnknown)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.GetLinkedVehicleByIDAndType(ctx, int32(1), entities.VehicleTypeTransporter)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestWateringPlanRepository_GetLinkedTreeClustersByID(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")

	t.Run("should return treecluster by watering plan id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		shouldReturn := allTestClusters[0:2]

		// when
		got, err := r.GetLinkedTreeClustersByID(context.Background(), 1)

		// then
		assert.NoError(t, err)
		assert.Len(t, got, len(shouldReturn))
		for i, tc := range got {
			assert.Equal(t, shouldReturn[i].ID, tc.ID)
			assert.Equal(t, shouldReturn[i].Name, tc.Name)
		}
	})

	t.Run("should return empty list when watering plan is not found", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedTreeClustersByID(context.Background(), 99)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should return empty list when watering plan with negative id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedTreeClustersByID(context.Background(), -1)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should return empty list when watering plan with zero id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedTreeClustersByID(context.Background(), 0)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.GetLinkedTreeClustersByID(ctx, 1)

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
	})
}

func TestWateringPlanRepository_GetLinkedUsersByID(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")

	t.Run("should return users by watering plan id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedUsersByID(context.Background(), 1)

		// then
		assert.NoError(t, err)
		assert.Len(t, got, 3)
		for _, userID := range got {
			assert.NotNil(t, userID)
		}
	})

	t.Run("should return empty list when watering plan is not found", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedUsersByID(context.Background(), 99)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should return empty list when watering plan with negative id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedUsersByID(context.Background(), -1)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should return empty list when watering plan with zero id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetLinkedUsersByID(context.Background(), 0)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.GetLinkedUsersByID(ctx, 1)

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
	})
}

func TestWateringPlanRepository_GetEvaluationValues(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")

	t.Run("should return evaluation values by watering plan id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		shouldReturn := allTestWateringPlans[2].Evaluation

		// when
		got, err := r.GetEvaluationValues(context.Background(), 3)

		// then
		assert.NoError(t, err)
		assert.Len(t, got, len(shouldReturn))
		for i, value := range got {
			assert.Equal(t, shouldReturn[i].WateringPlanID, value.WateringPlanID)
			assert.Equal(t, shouldReturn[i].TreeClusterID, value.TreeClusterID)
			assert.Equal(t, shouldReturn[i].ConsumedWater, value.ConsumedWater)
		}
	})

	t.Run("should return empty list when watering plan is not found", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetEvaluationValues(context.Background(), 99)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should return empty list when watering plan with negative id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetEvaluationValues(context.Background(), -1)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should return empty list when watering plan with zero id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.GetEvaluationValues(context.Background(), 0)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.GetEvaluationValues(ctx, 1)

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
	})
}

func TestWateringPlanRepository_GetAllUserCount(t *testing.T) {
	t.Run("should return count of all user linked to a watering plan in db", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")
		r := NewWateringPlanRepository(suite.Store, mappers)

		expectedCount := countUsersForWateringPlans()

		// when
		userCount, err := r.GetAllUserCount(context.Background())

		// then
		assert.NoError(t, err)
		assert.Equal(t, int64(expectedCount), userCount)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		userCount, err := r.GetAllUserCount(ctx)

		// then
		assert.Error(t, err)
		assert.Equal(t, int64(0), userCount)
	})
}

var allTestWateringPlans = []*entities.WateringPlan{
	{
		ID:                 1,
		Date:               time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
		Description:        "New watering plan for the west side of the city",
		Status:             entities.WateringPlanStatusPlanned,
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(720.0),
		Transporter:        allTestVehicles[1],
		Trailer:            allTestVehicles[0],
		TreeClusters:       allTestClusters[0:2],
		CancellationNote:   "",
		UserIDs:            parseUUIDs([]string{"6a1078e8-80fd-458f-b74e-e388fe2dd6ab", "05c028d9-62ef-4dcc-aa79-6b2fe9ce6f42", "e5ed176c-3aa8-4676-8e5b-0a0001a1bb88"}),
	},
	{
		ID:                 2,
		Date:               time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
		Description:        "New watering plan for the east side of the city",
		Status:             entities.WateringPlanStatusActive,
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(0.0),
		Transporter:        allTestVehicles[1],
		Trailer:            nil,
		TreeClusters:       allTestClusters[2:3],
		CancellationNote:   "",
		UserIDs:            parseUUIDs([]string{"6a1078e8-80fd-458f-b74e-e388fe2dd6ab"}),
		Provider:           "test-provider",
		AdditionalInfo: map[string]interface{}{
			"foo": "bar",
		},
	},
	{
		ID:                 3,
		Date:               time.Date(2024, 6, 12, 0, 0, 0, 0, time.UTC),
		Description:        "Very important watering plan due to no rainfall",
		Status:             entities.WateringPlanStatusFinished,
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(0.0),
		Transporter:        allTestVehicles[1],
		Trailer:            nil,
		TreeClusters:       allTestClusters[0:3],
		CancellationNote:   "",
		UserIDs:            parseUUIDs([]string{"6a1078e8-80fd-458f-b74e-e388fe2dd6ab"}),
		Evaluation: []*entities.EvaluationValue{
			{
				WateringPlanID: 3,
				TreeClusterID:  1,
				ConsumedWater:  utils.P(10.0),
			},
			{
				WateringPlanID: 3,
				TreeClusterID:  2,
				ConsumedWater:  utils.P(10.0),
			},
			{
				WateringPlanID: 3,
				TreeClusterID:  3,
				ConsumedWater:  utils.P(10.0),
			},
		},
	},
	{
		ID:                 4,
		Date:               time.Date(2024, 6, 10, 0, 0, 0, 0, time.UTC),
		Description:        "New watering plan for the south side of the city",
		Status:             entities.WateringPlanStatusNotCompeted,
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(0.0),
		Transporter:        allTestVehicles[1],
		Trailer:            nil,
		TreeClusters:       allTestClusters[2:3],
		CancellationNote:   "",
		UserIDs:            parseUUIDs([]string{"6a1078e8-80fd-458f-b74e-e388fe2dd6ab"}),
	},
	{
		ID:                 5,
		Date:               time.Date(2024, 6, 4, 0, 0, 0, 0, time.UTC),
		Description:        "Canceled due to flood",
		Status:             entities.WateringPlanStatusCanceled,
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(0.0),
		Transporter:        allTestVehicles[1],
		TreeClusters:       allTestClusters[2:3],
		CancellationNote:   "The watering plan was cancelled due to various reasons.",
		UserIDs:            parseUUIDs([]string{"6a1078e8-80fd-458f-b74e-e388fe2dd6ab"}),
	},
}

var allTestVehicles = []*entities.Vehicle{
	{
		ID:            1,
		NumberPlate:   "B-1234",
		Description:   "Test vehicle 1",
		WaterCapacity: 100.0,
		Type:          entities.VehicleTypeTrailer,
		Status:        entities.VehicleStatusActive,
	},
	{
		ID:            2,
		NumberPlate:   "B-5678",
		Description:   "Test vehicle 2",
		WaterCapacity: 150.0,
		Type:          entities.VehicleTypeTransporter,
		Status:        entities.VehicleStatusUnknown,
	},
}

var allTestClusters = []*entities.TreeCluster{
	{
		ID:             1,
		Name:           "Flensburger Stadion",
		WateringStatus: entities.WateringStatusGood,
		MoistureLevel:  0.75,
		Region: &entities.Region{
			ID:   1,
			Name: "Mürwik",
		},
		Address:       "Am Stadion",
		Description:   "Alle Bäume am Stadion",
		SoilCondition: entities.TreeSoilConditionSandig,
		Latitude:      utils.P(54.820940),
		Longitude:     utils.P(9.489022),
		Trees: []*entities.Tree{
			{ID: 1},
			{ID: 2},
			{ID: 3},
		},
	},
	{
		ID:             2,
		Name:           "Sankt-Jürgen-Platz",
		WateringStatus: entities.WateringStatusModerate,
		MoistureLevel:  0.5,
		Region: &entities.Region{
			ID:   1,
			Name: "Mürwik",
		},
		Address:       "Ulmenstraße",
		Description:   "Bäume beim Sankt-Jürgen-Platz",
		SoilCondition: entities.TreeSoilConditionSchluffig,
		Latitude:      utils.P(54.78805731048199),
		Longitude:     utils.P(9.44400186680097),
		Trees: []*entities.Tree{
			{ID: 4},
			{ID: 5},
			{ID: 6},
		},
	},
	{
		ID:             3,
		Name:           "Solitüde Strand",
		WateringStatus: "unknown",
		MoistureLevel:  0.7,
		Region: &entities.Region{
			ID:   1,
			Name: "Mürwik",
		},
		Address:       "Solitüde Strand",
		Description:   "Alle Bäume am Strand",
		SoilCondition: "schluffig",
		Latitude:      utils.P(54.802163),
		Longitude:     utils.P(9.446398),
		Trees:         []*entities.Tree{},
	},
}

func parseUUIDs(uuids []string) []*uuid.UUID {
	var parsedUUIDs []*uuid.UUID
	for _, u := range uuids {
		parsedUUID, err := uuid.Parse(u)
		if err != nil {
			return []*uuid.UUID{}
		}
		parsedUUIDs = append(parsedUUIDs, &parsedUUID)
	}

	return parsedUUIDs
}

func calculateCountTotalWater() int {
	count := 0
	for _, plan := range allTestWateringPlans {
		count += len(plan.TreeClusters)
	}

	return count * 10
}

func countUsersForWateringPlans() int {
	totalUsersCount := 0

	for _, plan := range allTestWateringPlans {
		totalUsersCount += len(plan.UserIDs)
	}

	return totalUsersCount
}
