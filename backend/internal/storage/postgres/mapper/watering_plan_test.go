package mapper_test

import (
	"fmt"
	"testing"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/mapper/generated"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
)

func TestWateringPlanMapper_FromSql(t *testing.T) {
	wateringPlanMapper := &generated.InternalWateringPlanRepoMapperImpl{}

	t.Run("should convert from sql to entity", func(t *testing.T) {
		// given
		src := allTestWateringPlans[0]

		// when
		got, err := wateringPlanMapper.FromSql(src)

		// then
		assert.NotNil(t, got)
		assert.NoError(t, err)
		assert.Equal(t, src.ID, got.ID)
		assert.Equal(t, src.CreatedAt, got.CreatedAt)
		assert.Equal(t, src.UpdatedAt, got.UpdatedAt)
		assert.Equal(t, src.Date, got.Date)
		assert.Equal(t, src.Description, got.Description)
		assert.Equal(t, src.Distance, got.Distance)
		assert.Equal(t, src.TotalWaterRequired, got.TotalWaterRequired)
		assert.Equal(t, src.Status, sqlc.WateringPlanStatus(got.Status))
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src *sqlc.WateringPlan = nil

		// when
		got, err := wateringPlanMapper.FromSql(src)

		// then
		assert.Nil(t, got)
		assert.NoError(t, err)
	})
}

func TestWateringPlanMapper_FromSqlList(t *testing.T) {
	wateringPlanMapper := &generated.InternalWateringPlanRepoMapperImpl{}

	t.Run("should convert from sql slice to entity slice", func(t *testing.T) {
		// given
		src := allTestWateringPlans

		// when
		got, err := wateringPlanMapper.FromSqlList(src)

		// then
		assert.NotNil(t, got)
		assert.NoError(t, err)
		assert.Len(t, got, 5)

		for i, src := range src {
			assert.NotNil(t, got)
			assert.Equal(t, src.ID, got[i].ID)
			assert.Equal(t, src.CreatedAt, got[i].CreatedAt)
			assert.Equal(t, src.UpdatedAt, got[i].UpdatedAt)
			assert.Equal(t, src.Date, got[i].Date)
			assert.Equal(t, src.Description, got[i].Description)
			assert.Equal(t, src.Distance, got[i].Distance)
			assert.Equal(t, src.TotalWaterRequired, got[i].TotalWaterRequired)
			assert.Equal(t, src.Status, sqlc.WateringPlanStatus(got[i].Status))
		}
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src []*sqlc.WateringPlan = nil

		// when
		got, err := wateringPlanMapper.FromSqlList(src)

		// then
		assert.Nil(t, got)
		assert.NoError(t, err)
	})
}

var allTestWateringPlans = []*sqlc.WateringPlan{
	{
		ID:                 1,
		Date:               time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
		Description:        "New watering plan for the west side of the city",
		Status:             sqlc.WateringPlanStatus(entities.WateringPlanStatusPlanned),
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(6000.0),
	},
	{
		ID:                 2,
		Date:               time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
		Description:        "New watering plan for the east side of the city",
		Status:             sqlc.WateringPlanStatus(entities.WateringPlanStatusActive),
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(6000.0),
	},
	{
		ID:                 3,
		Date:               time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
		Description:        "Very important watering plan due to no rainfall",
		Status:             sqlc.WateringPlanStatus(entities.WateringPlanStatusFinished),
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(6000.0),
	},
	{
		ID:                 4,
		Date:               time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
		Description:        "New watering plan for the south side of the city",
		Status:             sqlc.WateringPlanStatus(entities.WateringPlanStatusNotCompeted),
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(6000.0),
	},
	{
		ID:                 5,
		Date:               time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
		Description:        "Canceled due to flood",
		Status:             sqlc.WateringPlanStatus(entities.WateringPlanStatusCanceled),
		Distance:           utils.P(63.0),
		TotalWaterRequired: utils.P(6000.0),
	},
}

func TestMapWateringPlanStatus(t *testing.T) {
	tests := []struct {
		input    sqlc.WateringPlanStatus
		expected entities.WateringPlanStatus
	}{
		{input: sqlc.WateringPlanStatusActive, expected: entities.WateringPlanStatusActive},
		{input: sqlc.WateringPlanStatusCanceled, expected: entities.WateringPlanStatusCanceled},
		{input: sqlc.WateringPlanStatusFinished, expected: entities.WateringPlanStatusFinished},
		{input: sqlc.WateringPlanStatusNotcompeted, expected: entities.WateringPlanStatusNotCompeted},
		{input: sqlc.WateringPlanStatusPlanned, expected: entities.WateringPlanStatusPlanned},
		{input: sqlc.WateringPlanStatusUnknown, expected: entities.WateringPlanStatusUnknown},
	}

	for _, test := range tests {
		t.Run(fmt.Sprintf("should return %v for input %v", test.expected, test.input), func(t *testing.T) {
			result := mapper.MapWateringPlanStatus(test.input)
			assert.Equal(t, test.expected, result)
		})
	}
}
