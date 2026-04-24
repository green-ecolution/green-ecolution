package wateringplan

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/watering"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func TestWateringPlanRepository_Create(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/watering_plan")

	vehicleCount, _ := suite.Store.GetAllVehiclesCount(context.Background(), "")
	testVehicles, err := suite.Store.GetAllVehicles(context.Background(), &sqlc.GetAllVehiclesParams{
		Provider: "",
		Limit:    int32(vehicleCount),
		Offset:   0,
	})

	if err != nil {
		t.Fatal(err)
	}

	testCluster, err := suite.Store.GetAllTreeClusters(context.Background(), &sqlc.GetAllTreeClustersParams{
		Offset: 0,
		Limit:  5,
	})

	if err != nil {
		t.Fatal(err)
	}

	// UUID from test user in keycloak
	testUUID, err := uuid.Parse("6a1078e8-80fd-458f-b74e-e388fe2dd6ab")
	if err != nil {
		t.Fatal(err)
	}

	// Keep vehicle/cluster mappers for reference in test data lookups
	_, err = mappers.vehicleMapper.FromSqlList(testVehicles)
	if err != nil {
		t.Fatal(err)
	}
	_, err = mappers.clusterMapper.FromSqlList(testCluster)
	if err != nil {
		t.Fatal(err)
	}

	trailerID := testVehicles[2].ID
	transporterID := testVehicles[0].ID
	clusterIDs := make([]int32, 0, 3)
	for _, tc := range testCluster[0:3] {
		clusterIDs = append(clusterIDs, tc.ID)
	}

	input := watering.WateringPlan{
		Date:           time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
		Description:    "New watering plan",
		Distance:       utils.P(shared.MustNewDistance(50.0)),
		TrailerID:      &trailerID,
		TransporterID:  &transporterID,
		TreeClusterIDs: clusterIDs,
		UserIDs:        []*uuid.UUID{&testUUID},
	}

	expectedTotalWater := 720.0

	t.Run("should create watering plan with all values", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		createFn := func(wp *watering.WateringPlan, _ watering.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Description = input.Description
			wp.Distance = input.Distance
			wp.TransporterID = input.TransporterID
			wp.TrailerID = input.TrailerID
			wp.TreeClusterIDs = input.TreeClusterIDs
			wp.UserIDs = input.UserIDs
			wp.TotalWaterRequired = &expectedTotalWater
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.Equal(t, input.Date, got.Date)
		assert.Equal(t, input.Description, got.Description)
		assert.Equal(t, input.Distance, got.Distance)
		assert.Equal(t, expectedTotalWater, *got.TotalWaterRequired)
		assert.Equal(t, watering.WateringPlanStatusPlanned, got.Status)
		assert.Equal(t, "", got.CancellationNote)
		assert.Equal(t, 0, len(got.Evaluation))

		getWp, getErr := r.GetByID(context.Background(), got.ID)
		assert.NoError(t, getErr)

		// assert transporter
		assert.NotNil(t, getWp.TransporterID)
		assert.Equal(t, *input.TransporterID, *getWp.TransporterID)

		// assert trailer
		assert.NotNil(t, getWp.TrailerID)
		assert.Equal(t, *input.TrailerID, *getWp.TrailerID)

		// assert treecluster
		assert.Len(t, input.TreeClusterIDs, len(getWp.TreeClusterIDs))
		for i, tcID := range getWp.TreeClusterIDs {
			assert.Equal(t, input.TreeClusterIDs[i], tcID)
		}

		// assert user
		assert.Len(t, input.UserIDs, len(getWp.UserIDs))
		for i, userID := range getWp.UserIDs {
			assert.Equal(t, input.UserIDs[i], userID)
		}
	})

	t.Run("should create watering plan with default values", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		createFn := func(wp *watering.WateringPlan, _ watering.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.TransporterID = input.TransporterID
			wp.TreeClusterIDs = input.TreeClusterIDs
			wp.UserIDs = input.UserIDs
			wp.TotalWaterRequired = &expectedTotalWater
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.Equal(t, input.Date, got.Date)
		assert.Equal(t, "", got.Description)
		assert.Equal(t, utils.P(shared.MustNewDistance(0)), got.Distance)
		assert.Equal(t, expectedTotalWater, *got.TotalWaterRequired)
		assert.Equal(t, watering.WateringPlanStatusPlanned, got.Status)
		assert.Equal(t, "", got.CancellationNote)
		assert.Equal(t, 0, len(got.Evaluation))

		getWp, getErr := r.GetByID(context.Background(), got.ID)
		assert.NoError(t, getErr)

		// assert transporter
		assert.NotNil(t, getWp.TransporterID)
		assert.Equal(t, *input.TransporterID, *getWp.TransporterID)

		// assert no trailer
		assert.Nil(t, got.TrailerID)

		// assert treecluster
		assert.Len(t, input.TreeClusterIDs, len(getWp.TreeClusterIDs))
		for i, tcID := range getWp.TreeClusterIDs {
			assert.Equal(t, input.TreeClusterIDs[i], tcID)
		}

		// assert user
		assert.Len(t, input.UserIDs, len(getWp.UserIDs))
		for i, userID := range getWp.UserIDs {
			assert.Equal(t, input.UserIDs[i], userID)
		}
	})

	t.Run("should return error when date is not in correct format", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		createFn := func(wp *watering.WateringPlan, _ watering.WateringPlanRepository) (bool, error) {
			wp.Date = time.Time{}
			wp.TransporterID = input.TransporterID
			wp.TreeClusterIDs = input.TreeClusterIDs
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, "failed to convert date", err.Error())
	})

	t.Run("should return error when watering plan has no linked users", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		createFn := func(wp *watering.WateringPlan, _ watering.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.TransporterID = input.TransporterID
			wp.TrailerID = input.TrailerID
			wp.TreeClusterIDs = input.TreeClusterIDs
			wp.UserIDs = []*uuid.UUID{}
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, "watering plan requires employees", err.Error())
	})

	t.Run("should return error when transporter is nil", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		createFn := func(wp *watering.WateringPlan, _ watering.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.TransporterID = nil
			wp.TrailerID = input.TrailerID
			wp.TreeClusterIDs = input.TreeClusterIDs
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, "watering plan requires a valid transporter", err.Error())
	})

	t.Run("should return error when no treecluster are linked", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		createFn := func(wp *watering.WateringPlan, _ watering.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.TransporterID = input.TransporterID
			wp.TrailerID = input.TrailerID
			wp.TreeClusterIDs = []int32{}
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, "watering plan requires tree cluster", err.Error())
	})

	t.Run("should return error when watering plan is invalid", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.Create(context.Background(), nil)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, "createFn is nil", err.Error())
	})

	t.Run("should return error if context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		createFn := func(wp *watering.WateringPlan, _ watering.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.TransporterID = input.TransporterID
			wp.TreeClusterIDs = input.TreeClusterIDs
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.Create(ctx, createFn)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when createFn returns error", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		createFn := func(wp *watering.WateringPlan, _ watering.WateringPlanRepository) (bool, error) {
			return false, assert.AnError
		}

		wp, err := r.Create(context.Background(), createFn)
		assert.Error(t, err)
		assert.Nil(t, wp)
	})

	t.Run("should not create watering plan when createFn returns false", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		createFn := func(wp *watering.WateringPlan, _ watering.WateringPlanRepository) (bool, error) {
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

		r := NewWateringPlanRepository(suite.Store, mappers)
		createFn := func(wp *watering.WateringPlan, _ watering.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.TransporterID = input.TransporterID
			wp.TrailerID = input.TrailerID
			wp.TreeClusterIDs = input.TreeClusterIDs
			wp.UserIDs = input.UserIDs
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
