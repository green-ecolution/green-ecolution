package info

import (
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/stretchr/testify/assert"
)

func TestGetMapInfo(t *testing.T) {
	t.Run("should return map info with valid config", func(t *testing.T) {
		// given
		cfg := &config.Config{
			Map: config.MapConfig{
				Center: []float64{54.0, 10.0},
				BBox:   []float64{9.0, 53.0, 11.0, 55.0},
			},
		}

		// when
		result, err := getMapInfo(cfg)

		// then
		assert.NoError(t, err)
		assert.Equal(t, []float64{54.0, 10.0}, result.Center)
		assert.Equal(t, []float64{9.0, 53.0, 11.0, 55.0}, result.BBox)
	})

	t.Run("should return error when center has wrong length", func(t *testing.T) {
		// given
		cfg := &config.Config{
			Map: config.MapConfig{
				Center: []float64{54.0}, // only 1 element instead of 2
				BBox:   []float64{9.0, 53.0, 11.0, 55.0},
			},
		}

		// when
		_, err := getMapInfo(cfg)

		// then
		assert.ErrorIs(t, err, storage.ErrInvalidMapConfig)
	})

	t.Run("should return error when center is empty", func(t *testing.T) {
		// given
		cfg := &config.Config{
			Map: config.MapConfig{
				Center: []float64{},
				BBox:   []float64{9.0, 53.0, 11.0, 55.0},
			},
		}

		// when
		_, err := getMapInfo(cfg)

		// then
		assert.ErrorIs(t, err, storage.ErrInvalidMapConfig)
	})

	t.Run("should return error when center has too many elements", func(t *testing.T) {
		// given
		cfg := &config.Config{
			Map: config.MapConfig{
				Center: []float64{54.0, 10.0, 5.0}, // 3 elements instead of 2
				BBox:   []float64{9.0, 53.0, 11.0, 55.0},
			},
		}

		// when
		_, err := getMapInfo(cfg)

		// then
		assert.ErrorIs(t, err, storage.ErrInvalidMapConfig)
	})

	t.Run("should return error when bbox has wrong length", func(t *testing.T) {
		// given
		cfg := &config.Config{
			Map: config.MapConfig{
				Center: []float64{54.0, 10.0},
				BBox:   []float64{9.0, 53.0}, // only 2 elements instead of 4
			},
		}

		// when
		_, err := getMapInfo(cfg)

		// then
		assert.ErrorIs(t, err, storage.ErrInvalidMapConfig)
	})

	t.Run("should return error when bbox is empty", func(t *testing.T) {
		// given
		cfg := &config.Config{
			Map: config.MapConfig{
				Center: []float64{54.0, 10.0},
				BBox:   []float64{},
			},
		}

		// when
		_, err := getMapInfo(cfg)

		// then
		assert.ErrorIs(t, err, storage.ErrInvalidMapConfig)
	})

	t.Run("should return error when bbox has too many elements", func(t *testing.T) {
		// given
		cfg := &config.Config{
			Map: config.MapConfig{
				Center: []float64{54.0, 10.0},
				BBox:   []float64{9.0, 53.0, 11.0, 55.0, 12.0}, // 5 elements instead of 4
			},
		}

		// when
		_, err := getMapInfo(cfg)

		// then
		assert.ErrorIs(t, err, storage.ErrInvalidMapConfig)
	})

	t.Run("should return error when both center and bbox are invalid", func(t *testing.T) {
		// given
		cfg := &config.Config{
			Map: config.MapConfig{
				Center: []float64{54.0},
				BBox:   []float64{9.0, 53.0},
			},
		}

		// when
		_, err := getMapInfo(cfg)

		// then
		assert.ErrorIs(t, err, storage.ErrInvalidMapConfig)
	})
}
