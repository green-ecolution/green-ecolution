package region

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestRegionRepository_Create(t *testing.T) {
	suite.ResetDB(t)
	t.Run("should create region", func(t *testing.T) {
		// given
		r := NewRegionRepository(defaultFields.store, defaultFields.RegionMappers)

		// when
		got, err := r.Create(context.Background(), WithName("test"))

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, "test", got.Name)
	})

	t.Run("should return error when create region with empty name", func(t *testing.T) {
		// given
		r := NewRegionRepository(defaultFields.store, defaultFields.RegionMappers)

		// when
		got, err := r.Create(context.Background())

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
		got, err := r.Create(ctx, WithName("test"))

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}
