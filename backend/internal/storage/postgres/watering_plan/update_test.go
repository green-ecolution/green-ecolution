package wateringplan

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
)

func TestWateringPlanRepository_Update(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/watering_plan")

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

	vehicle, err := mappers.vehicleMapper.FromSqlList(testVehicles)
	if err != nil {
		t.Fatal(err)
	}
	treeClusters, err := mappers.clusterMapper.FromSqlList(testCluster)
	if err != nil {
		t.Fatal(err)
	}

	input := entities.WateringPlan{
		Date:         time.Date(2024, 11, 22, 0, 0, 0, 0, time.UTC),
		Description:  "Updated watering plan",
		Distance:     utils.P(50.0),
		Trailer:      vehicle[3],
		Transporter:  vehicle[1],
		TreeClusters: treeClusters[0:3],
		UserIDs:      []*uuid.UUID{&testUUID},
		Status:       entities.WateringPlanStatusActive,
	}

	expectedTotalWater := 720.0

	evaluation := []*entities.EvaluationValue{
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

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Description = input.Description
			wp.Distance = input.Distance
			wp.Transporter = input.Transporter
			wp.Trailer = input.Trailer
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			wp.Status = input.Status
			return true, nil
		}

		// when
		updateErr := r.Update(context.Background(), 1, updateFn)
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
		assert.Equal(t, input.Transporter.ID, got.Transporter.ID)
		assert.Equal(t, input.Transporter.NumberPlate, got.Transporter.NumberPlate)

		// assert trailer
		assert.Equal(t, input.Trailer.ID, got.Trailer.ID)
		assert.Equal(t, input.Trailer.NumberPlate, got.Trailer.NumberPlate)

		// assert TreeClusters
		assert.Len(t, input.TreeClusters, len(got.TreeClusters))
		for i, tc := range got.TreeClusters {
			assert.Equal(t, input.TreeClusters[i].ID, tc.ID)
			assert.Equal(t, input.TreeClusters[i].Name, tc.Name)
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

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Description = input.Description
			wp.Distance = input.Distance
			wp.Transporter = input.Transporter
			wp.Trailer = nil
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			wp.Status = input.Status
			wp.TotalWaterRequired = &expectedTotalWater
			return true, nil
		}

		// when
		updateErr := r.Update(context.Background(), 2, updateFn)
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
		assert.Equal(t, input.Transporter.ID, got.Transporter.ID)
		assert.Equal(t, input.Transporter.NumberPlate, got.Transporter.NumberPlate)

		// assert nil trailer
		assert.Nil(t, got.Trailer)

		// assert TreeClusters
		assert.Len(t, input.TreeClusters, len(got.TreeClusters))
		for i, tc := range got.TreeClusters {
			assert.Equal(t, input.TreeClusters[i].ID, tc.ID)
			assert.Equal(t, input.TreeClusters[i].Name, tc.Name)
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

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Description = input.Description
			wp.Distance = input.Distance
			wp.Transporter = input.Transporter
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			wp.Status = entities.WateringPlanStatusCanceled
			wp.CancellationNote = cancellationNote
			return true, nil
		}

		// when
		updateErr := r.Update(context.Background(), 2, updateFn)
		got, getErr := r.GetByID(context.Background(), 2)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.Equal(t, input.Date, got.Date)
		assert.Equal(t, entities.WateringPlanStatusCanceled, got.Status)
		assert.Equal(t, cancellationNote, got.CancellationNote)
	})

	t.Run("should not update consumed water values if status is not finished", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Distance = input.Distance
			wp.Transporter = input.Transporter
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			wp.Status = entities.WateringPlanStatusNotCompeted
			wp.Evaluation = evaluation
			return true, nil
		}

		// when
		updateErr := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.Equal(t, entities.WateringPlanStatusNotCompeted, got.Status)

		// assert consumed water list
		gotEvaluation, err := r.GetEvaluationValues(context.Background(), 1)
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

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Description = input.Description
			wp.Distance = input.Distance
			wp.Transporter = input.Transporter
			wp.Trailer = input.Trailer
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			wp.Status = entities.WateringPlanStatusFinished
			wp.Evaluation = evaluation
			return true, nil
		}

		// when
		updateErr := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.Equal(t, entities.WateringPlanStatusFinished, got.Status)

		// assert consumed water list
		gotEvaluation, err := r.GetEvaluationValues(context.Background(), 1)
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

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Distance = input.Distance
			wp.Transporter = input.Transporter
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			wp.Status = entities.WateringPlanStatusActive
			wp.CancellationNote = "This watering plan is canceled"
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 2, updateFn)

		// then
		assert.Error(t, err)
		assert.Equal(t, "cancellation note should be empty, as the current watering plan is not canceled", err.Error())
	})

	t.Run("should return error when date is not in correct format", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = time.Time{}
			wp.Transporter = input.Transporter
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
		assert.Equal(t, "failed to convert date", err.Error())
	})

	t.Run("should return error when trailer vehicle has not correct vehilce type", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Transporter = input.Transporter
			wp.Trailer = input.Transporter
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
		assert.Equal(t, "trailer vehicle requires a vehicle of type trailer", err.Error())
	})

	t.Run("should return error when watering plan has no linked users", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Transporter = input.Transporter
			wp.Trailer = input.Trailer
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = []*uuid.UUID{}
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
		assert.Equal(t, "watering plan requires employees", err.Error())
	})

	t.Run("should return error when transporter has not correct vehilce type", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Transporter = input.Trailer
			wp.Trailer = input.Trailer
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
		assert.Equal(t, "watering plan requires a valid transporter", err.Error())
	})

	t.Run("should return error when transporter is nil", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Transporter = nil
			wp.Trailer = input.Trailer
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
		assert.Equal(t, "watering plan requires a valid transporter", err.Error())
	})

	t.Run("should return error when no TreeClusters are linked", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Transporter = input.Transporter
			wp.Trailer = input.Trailer
			wp.TreeClusters = []*entities.TreeCluster{}
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
		assert.Equal(t, "watering plan requires tree cluster", err.Error())
	})

	t.Run("should return error when watering plan is invalid", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		err := r.Update(context.Background(), 1, nil)

		// then
		assert.Error(t, err)
		assert.Equal(t, "updateFn is nil", err.Error())
	})

	t.Run("should return error when update watering plan with negative id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Transporter = input.Transporter
			wp.Trailer = input.Trailer
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		// when
		err := r.Update(context.Background(), -1, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when update watering plan with zero id", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Transporter = input.Transporter
			wp.Trailer = input.Trailer
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 0, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when update watering plan not found", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Transporter = input.Transporter
			wp.Trailer = input.Trailer
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 99, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error if context is canceled", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Transporter = input.Transporter
			wp.Trailer = input.Trailer
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			return true, nil
		}

		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		err := r.Update(ctx, 99, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when updateFn is nil", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)

		// when
		err := r.Update(context.Background(), 1, nil)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when updateFn returns error", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			return true, assert.AnError
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should not update when updateFn returns false", func(t *testing.T) {
		// given
		r := NewWateringPlanRepository(suite.Store, mappers)
		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
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
		r := NewWateringPlanRepository(suite.Store, mappers)
		updateFn := func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
			wp.Date = input.Date
			wp.Description = "Test"
			wp.Transporter = input.Transporter
			wp.Trailer = input.Trailer
			wp.TreeClusters = input.TreeClusters
			wp.UserIDs = input.UserIDs
			return false, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, err)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotEqual(t, "Test", got.Description)
	})
}
