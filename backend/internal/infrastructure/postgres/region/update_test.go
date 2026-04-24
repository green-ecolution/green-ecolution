package region

import (
	"context"
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/region"
	"github.com/stretchr/testify/assert"
)

func TestRegionRepository_Update(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/region")

	t.Run("should update region", func(t *testing.T) {
		// given
		r := NewRegionRepository(defaultFields.store, defaultFields.RegionMappers)

		// when
		got, err := r.Update(context.Background(), 1, &region.Region{Name: "test"})
		gotByID, _ := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, "test", got.Name)
		assert.Equal(t, "test", gotByID.Name)
	})

	t.Run("should return error when update region with empty name", func(t *testing.T) {
		// given
		r := NewRegionRepository(defaultFields.store, defaultFields.RegionMappers)

		// when
		got, err := r.Update(context.Background(), 2, &region.Region{Name: ""})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when update region with negative id", func(t *testing.T) {
		// given
		r := NewRegionRepository(defaultFields.store, defaultFields.RegionMappers)

		// when
		got, err := r.Update(context.Background(), -1, &region.Region{Name: "test"})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when update region with zero id", func(t *testing.T) {
		// given
		r := NewRegionRepository(defaultFields.store, defaultFields.RegionMappers)

		// when
		got, err := r.Update(context.Background(), 0, &region.Region{})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when update region not found", func(t *testing.T) {
		// given
		r := NewRegionRepository(defaultFields.store, defaultFields.RegionMappers)

		// when
		got, err := r.Update(context.Background(), 99, &region.Region{Name: "test"})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewRegionRepository(defaultFields.store, defaultFields.RegionMappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.Update(ctx, 1, &region.Region{Name: "test"})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}
