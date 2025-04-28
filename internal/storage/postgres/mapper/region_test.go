package mapper_test

import (
	"testing"
	"time"

	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/backend/internal/storage/postgres/mapper/generated"
	"github.com/jackc/pgx/v5/pgtype"
	"github.com/stretchr/testify/assert"
)

func TestRegionMapper_FromSql(t *testing.T) {
	regionMapper := &generated.InternalRegionRepoMapperImpl{}

	t.Run("should convert from sql to entity", func(t *testing.T) {
		// given
		src := allTestRegions[0]

		// when
		got := regionMapper.FromSql(src)

		// then
		assert.NotNil(t, got)
		assert.Equal(t, src.ID, got.ID)
		assert.Equal(t, src.CreatedAt.Time, got.CreatedAt)
		assert.Equal(t, src.UpdatedAt.Time, got.UpdatedAt)
		assert.Equal(t, src.Name, got.Name)
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src *sqlc.Region = nil

		// when
		got := regionMapper.FromSql(src)

		// then
		assert.Nil(t, got)
	})
}

func TestRegionMapper_FromSqlList(t *testing.T) {
	regionMapper := &generated.InternalRegionRepoMapperImpl{}

	t.Run("should convert from sql slice to entity slice", func(t *testing.T) {
		// given
		src := allTestRegions

		// when
		got := regionMapper.FromSqlList(src)

		// then
		assert.NotNil(t, got)
		assert.Len(t, got, 2)

		for i, src := range src {
			assert.NotNil(t, got)
			assert.Equal(t, src.ID, got[i].ID)
			assert.Equal(t, src.CreatedAt.Time, got[i].CreatedAt)
			assert.Equal(t, src.UpdatedAt.Time, got[i].UpdatedAt)
			assert.Equal(t, src.Name, got[i].Name)
		}
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src []*sqlc.Region = nil

		// when
		got := regionMapper.FromSqlList(src)

		// then
		assert.Nil(t, got)
	})
}

var allTestRegions = []*sqlc.Region{
	{
		ID:        1,
		CreatedAt: pgtype.Timestamp{Time: time.Now()},
		UpdatedAt: pgtype.Timestamp{Time: time.Now()},
		Name:      "Mürwik",
	},
	{
		ID:        1,
		CreatedAt: pgtype.Timestamp{Time: time.Now()},
		UpdatedAt: pgtype.Timestamp{Time: time.Now()},
		Name:      "Innenstadt",
	},
}
