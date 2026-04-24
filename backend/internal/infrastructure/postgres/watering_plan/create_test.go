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

	vehicleCount, _ := suite.Store.GetAllVehiclesCount(context.Background(), &sqlc.GetAllVehiclesCountParams{})
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
		entity := input
		entity.TotalWaterRequired = &expectedTotalWater

		// when
		got, err := r.Create(context.Background(), &entity)

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
		entity := watering.WateringPlan{
			Date:               input.Date,
			TransporterID:      input.TransporterID,
			TreeClusterIDs:     input.TreeClusterIDs,
			UserIDs:            input.UserIDs,
			TotalWaterRequired: &expectedTotalWater,
		}

		// when
		got, err := r.Create(context.Background(), &entity)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.Equal(t, input.Date, got.Date)
		assert.Equal(t, "", got.Description)
		assert.Nil(t, got.Distance)
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
		entity := input
		entity.Date = time.Time{} // zero time

		// when
		got, err := r.Create(context.Background(), &entity)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, "failed to convert date", err.Error())
	})

	t.Run("should return error when watering plan has no linked users", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.UserIDs = nil

		// when
		got, err := r.Create(context.Background(), &entity)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, "watering plan requires employees", err.Error())
	})

	t.Run("should return error when transporter is nil", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.TransporterID = nil

		// when
		got, err := r.Create(context.Background(), &entity)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, "watering plan requires a valid transporter", err.Error())
	})

	t.Run("should return error when no treecluster are linked", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.TreeClusterIDs = nil

		// when
		got, err := r.Create(context.Background(), &entity)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, "watering plan requires tree cluster", err.Error())
	})

	t.Run("should return error when entity is nil", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		got, err := r.Create(context.Background(), nil)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		assert.Equal(t, "entity is nil", err.Error())
	})

	t.Run("should return error if context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		ctx, cancel := context.WithCancel(context.Background())
		cancel()
		entity := input

		// when
		got, err := r.Create(ctx, &entity)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}
