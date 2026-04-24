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

func TestWateringPlanRepository_Update(t *testing.T) {
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

	vehicles, err := mappers.vehicleMapper.FromSqlList(testVehicles)
	if err != nil {
		t.Fatal(err)
	}
	_, err = mappers.clusterMapper.FromSqlList(testCluster)
	if err != nil {
		t.Fatal(err)
	}

	trailerID := vehicles[3].ID
	transporterID := vehicles[1].ID
	clusterIDs := make([]int32, 0, 3)
	for _, tc := range testCluster[0:3] {
		clusterIDs = append(clusterIDs, tc.ID)
	}

	input := watering.WateringPlan{
		Date:           time.Date(2024, 11, 22, 0, 0, 0, 0, time.UTC),
		Description:    "Updated watering plan",
		Distance:       utils.P(shared.MustNewDistance(50.0)),
		TrailerID:      &trailerID,
		TransporterID:  &transporterID,
		TreeClusterIDs: clusterIDs,
		UserIDs:        []*uuid.UUID{&testUUID},
		Status:         watering.WateringPlanStatusActive,
	}

	expectedTotalWater := 720.0

	evaluation := []*watering.EvaluationValue{
		{
			WateringPlanID: 1,
			TreeClusterID:  1,
			ConsumedWater:  utils.P(10.0),
		},
		{
			WateringPlanID: 1,
			TreeClusterID:  2,
			ConsumedWater:  utils.P(10.0),
		},
		{
			WateringPlanID: 1,
			TreeClusterID:  3,
			ConsumedWater:  utils.P(10.0),
		},
	}

	t.Run("should update watering plan", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.TotalWaterRequired = &expectedTotalWater

		// when
		updateErr := r.Update(context.Background(), 1, &entity)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.Equal(t, input.Date, got.Date)
		assert.Equal(t, input.Description, got.Description)
		assert.Equal(t, input.Distance, got.Distance)
		assert.Equal(t, expectedTotalWater, *got.TotalWaterRequired)
		assert.Equal(t, input.Status, got.Status)

		// assert transporter
		assert.Equal(t, *input.TransporterID, *got.TransporterID)

		// assert trailer
		assert.Equal(t, *input.TrailerID, *got.TrailerID)

		// assert TreeClusters
		assert.Len(t, input.TreeClusterIDs, len(got.TreeClusterIDs))
		for i, tc := range got.TreeClusterIDs {
			assert.Equal(t, input.TreeClusterIDs[i], tc)
		}

		// assert user
		assert.Len(t, input.UserIDs, len(got.UserIDs))
		for i, userID := range got.UserIDs {
			assert.Equal(t, input.UserIDs[i], userID)
		}
	})

	t.Run("should update watering plan and unlink trailer", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.TrailerID = nil
		entity.TotalWaterRequired = &expectedTotalWater

		// when
		updateErr := r.Update(context.Background(), 2, &entity)
		got, getErr := r.GetByID(context.Background(), 2)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.Equal(t, input.Date, got.Date)
		assert.Equal(t, input.Description, got.Description)
		assert.Equal(t, input.Distance, got.Distance)
		assert.Equal(t, expectedTotalWater, *got.TotalWaterRequired)
		assert.Equal(t, input.Status, got.Status)

		// assert transporter
		assert.Equal(t, *input.TransporterID, *got.TransporterID)

		// assert nil trailer
		assert.Nil(t, got.TrailerID)

		// assert TreeClusters
		assert.Len(t, input.TreeClusterIDs, len(got.TreeClusterIDs))
		for i, tc := range got.TreeClusterIDs {
			assert.Equal(t, input.TreeClusterIDs[i], tc)
		}

		// assert user
		assert.Len(t, input.UserIDs, len(got.UserIDs))
		for i, userID := range got.UserIDs {
			assert.Equal(t, input.UserIDs[i], userID)
		}
	})

	t.Run("should update watering plan to canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		cancellationNote := "This watering plan is canceled"
		entity := input
		entity.Status = watering.WateringPlanStatusCanceled
		entity.CancellationNote = cancellationNote
		entity.TotalWaterRequired = &expectedTotalWater

		// when
		updateErr := r.Update(context.Background(), 2, &entity)
		got, getErr := r.GetByID(context.Background(), 2)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.Equal(t, input.Date, got.Date)
		assert.Equal(t, watering.WateringPlanStatusCanceled, got.Status)
		assert.Equal(t, cancellationNote, got.CancellationNote)
	})

	t.Run("should not update consumed water values if status is not finished", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.Status = watering.WateringPlanStatusNotCompeted
		entity.Evaluation = evaluation
		entity.TotalWaterRequired = &expectedTotalWater

		// when
		updateErr := r.Update(context.Background(), 1, &entity)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.Equal(t, watering.WateringPlanStatusNotCompeted, got.Status)

		// assert consumed water list
		gotEvaluation, err := r.getEvaluationValues(context.Background(), 1)
		assert.NoError(t, err)
		assert.NotNil(t, gotEvaluation)
		for i, evaluationValue := range gotEvaluation {
			assert.Equal(t, int32(1), evaluationValue.WateringPlanID)
			assert.Equal(t, evaluation[i].TreeClusterID, evaluationValue.TreeClusterID)
			assert.Equal(t, 0.0, *evaluationValue.ConsumedWater) // should be still zero due to no update
		}
	})

	t.Run("should update watering plan to finished and set consumed water values", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.Status = watering.WateringPlanStatusFinished
		entity.Evaluation = evaluation
		entity.TotalWaterRequired = &expectedTotalWater

		// when
		updateErr := r.Update(context.Background(), 1, &entity)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.Equal(t, watering.WateringPlanStatusFinished, got.Status)

		// assert consumed water list
		gotEvaluation, err := r.getEvaluationValues(context.Background(), 1)
		assert.NoError(t, err)
		assert.NotNil(t, gotEvaluation)
		assert.Len(t, gotEvaluation, len(evaluation))
		for i, evaluationValue := range evaluation {
			assert.Equal(t, int32(1), evaluationValue.WateringPlanID)
			assert.Equal(t, evaluation[i].TreeClusterID, evaluationValue.TreeClusterID)
			assert.Equal(t, evaluation[i].ConsumedWater, evaluationValue.ConsumedWater) // should be updated
		}
	})

	t.Run("should return error when cancellation note is not empty and the status is not canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.CancellationNote = "should fail"
		entity.Status = watering.WateringPlanStatusActive

		// when
		err := r.Update(context.Background(), 2, &entity)

		// then
		assert.Error(t, err)
		assert.Equal(t, "cancellation note should be empty, as the current watering plan is not canceled", err.Error())
	})

	t.Run("should return error when date is not in correct format", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.Date = time.Time{} // zero time

		// when
		err := r.Update(context.Background(), 1, &entity)

		// then
		assert.Error(t, err)
		assert.Equal(t, "failed to convert date", err.Error())
	})

	t.Run("should return error when trailer vehicle has not correct vehilce type", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.TrailerID = &transporterID // use transporter as trailer (wrong type)

		// when
		err := r.Update(context.Background(), 1, &entity)

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "duplicate key")
	})

	t.Run("should return error when watering plan has no linked users", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.UserIDs = nil

		// when
		err := r.Update(context.Background(), 1, &entity)

		// then
		assert.Error(t, err)
		assert.Equal(t, "watering plan requires employees", err.Error())
	})

	t.Run("should return error when transporter has not correct vehilce type", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.TransporterID = &trailerID // use trailer as transporter (wrong type)

		// when
		err := r.Update(context.Background(), 1, &entity)

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "duplicate key")
	})

	t.Run("should return error when transporter is nil", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.TransporterID = nil

		// when
		err := r.Update(context.Background(), 1, &entity)

		// then
		assert.Error(t, err)
		assert.Equal(t, "watering plan requires a valid transporter", err.Error())
	})

	t.Run("should return error when no TreeClusters are linked", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input
		entity.TreeClusterIDs = nil

		// when
		err := r.Update(context.Background(), 1, &entity)

		// then
		assert.Error(t, err)
		assert.Equal(t, "watering plan requires tree cluster", err.Error())
	})

	t.Run("should return error when entity is nil", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		err := r.Update(context.Background(), 1, nil)

		// then
		assert.Error(t, err)
		assert.Equal(t, "entity is nil", err.Error())
	})

	t.Run("should return error when update watering plan with negative id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input

		// when
		err := r.Update(context.Background(), -1, &entity)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when update watering plan with zero id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input

		// when
		err := r.Update(context.Background(), 0, &entity)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when update watering plan not found", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		entity := input

		// when
		err := r.Update(context.Background(), 99, &entity)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error if context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		ctx, cancel := context.WithCancel(context.Background())
		cancel()
		entity := input

		// when
		err := r.Update(ctx, 99, &entity)

		// then
		assert.Error(t, err)
	})
}
