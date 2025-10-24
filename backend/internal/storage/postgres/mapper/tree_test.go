package mapper_test

import (
	"testing"
	"time"

	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/backend/internal/storage/postgres/mapper/generated"
	"github.com/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
)

func TestTreeMapper_FromSql(t *testing.T) {
	treeMapper := &generated.InternalTreeRepoMapperImpl{}

	t.Run("should convert from sql to entity", func(t *testing.T) {
		// given
		src := allTestTrees[0]

		// when
		got, err := treeMapper.FromSql(src)

		// then
		assert.NotNil(t, got)
		assert.NoError(t, err)
		assert.Equal(t, src.ID, got.ID)
		assert.Equal(t, src.CreatedAt, got.CreatedAt)
		assert.Equal(t, src.UpdatedAt, got.UpdatedAt)
		assert.Equal(t, src.PlantingYear, got.PlantingYear)
		assert.Equal(t, src.Species, got.Species)
		assert.Equal(t, src.Number, got.Number)
		assert.Equal(t, src.Latitude, got.Latitude)
		assert.Equal(t, src.Longitude, got.Longitude)
		assert.Equal(t, src.WateringStatus, sqlc.WateringStatus(got.WateringStatus))
		assert.Equal(t, *src.Provider, got.Provider)
		assert.Equal(t, *src.Description, got.Description)
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src *sqlc.Tree = nil

		// when
		got, err := treeMapper.FromSql(src)

		// then
		assert.Nil(t, got)
		assert.NoError(t, err)
	})
}

func TestTreeMapper_FromSqlList(t *testing.T) {
	treeMapper := &generated.InternalTreeRepoMapperImpl{}

	t.Run("should convert from sql slice to entity slice", func(t *testing.T) {
		// given
		src := allTestTrees

		// when
		got, err := treeMapper.FromSqlList(src)

		// then
		assert.NotNil(t, got)
		assert.NoError(t, err)
		assert.Len(t, got, 2)

		for i, src := range src {
			assert.NotNil(t, got)
			assert.Equal(t, src.ID, got[i].ID)
			assert.Equal(t, src.CreatedAt, got[i].CreatedAt)
			assert.Equal(t, src.UpdatedAt, got[i].UpdatedAt)
			assert.Equal(t, src.PlantingYear, got[i].PlantingYear)
			assert.Equal(t, src.Species, got[i].Species)
			assert.Equal(t, src.Number, got[i].Number)
			assert.Equal(t, src.Latitude, got[i].Latitude)
			assert.Equal(t, src.Longitude, got[i].Longitude)
			assert.Equal(t, src.WateringStatus, sqlc.WateringStatus(got[i].WateringStatus))
			assert.Equal(t, *src.Provider, got[i].Provider)
			assert.Equal(t, *src.Description, got[i].Description)
		}
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src []*sqlc.Tree = nil

		// when
		got, err := treeMapper.FromSqlList(src)

		// then
		assert.Nil(t, got)
		assert.NoError(t, err)
	})
}

var allTestTrees = []*sqlc.Tree{
	{
		ID:             1,
		CreatedAt:      time.Now(),
		UpdatedAt:      time.Now(),
		PlantingYear:   2024,
		Species:        "Oak",
		Latitude:       52.5200,
		Longitude:      13.4050,
		WateringStatus: sqlc.WateringStatusGood,
		Description:    utils.P("Newly planted tree"),
		Number:         "P 1234",
		Provider:       utils.P(""),
	},
	{
		ID:             2,
		CreatedAt:      time.Now(),
		UpdatedAt:      time.Now(),
		PlantingYear:   2024,
		Species:        "Maple",
		Latitude:       52.5200,
		Longitude:      13.4050,
		WateringStatus: sqlc.WateringStatusModerate,
		Description:    utils.P("Also newly planted tree"),
		Number:         "P 2345",
		Provider:       utils.P("foo-provider"),
	},
}
