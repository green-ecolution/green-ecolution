package routing

import (
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestConvertLocations(t *testing.T) {
	t.Run("should convert valid configuration", func(t *testing.T) {
		// given
		cfg := &config.RoutingConfig{
			StartPoint:    []float64{9.0, 48.0},
			EndPoint:      []float64{9.1, 48.1},
			WateringPoint: []float64{9.2, 48.2},
		}

		// when
		result, err := ConvertLocations(cfg)

		// then
		require.NoError(t, err)
		assert.Equal(t, 9.0, result.StartPoint.Longitude)
		assert.Equal(t, 48.0, result.StartPoint.Latitude)
		assert.Equal(t, 9.1, result.EndPoint.Longitude)
		assert.Equal(t, 48.1, result.EndPoint.Latitude)
		assert.Equal(t, 9.2, result.WateringPoint.Longitude)
		assert.Equal(t, 48.2, result.WateringPoint.Latitude)
	})

	t.Run("should return error for invalid EndPoint with 1 element", func(t *testing.T) {
		// given
		cfg := &config.RoutingConfig{
			StartPoint:    []float64{9.0, 48.0},
			EndPoint:      []float64{9.1},
			WateringPoint: []float64{9.2, 48.2},
		}

		// when
		result, err := ConvertLocations(cfg)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "invalid EndPoint")
	})

	t.Run("should return error for invalid StartPoint with 0 elements", func(t *testing.T) {
		// given
		cfg := &config.RoutingConfig{
			StartPoint:    []float64{},
			EndPoint:      []float64{9.1, 48.1},
			WateringPoint: []float64{9.2, 48.2},
		}

		// when
		result, err := ConvertLocations(cfg)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "invalid StartPoint")
	})

	t.Run("should return error for invalid WateringPoint with 3 elements", func(t *testing.T) {
		// given
		cfg := &config.RoutingConfig{
			StartPoint:    []float64{9.0, 48.0},
			EndPoint:      []float64{9.1, 48.1},
			WateringPoint: []float64{9.2, 48.2, 0.0},
		}

		// when
		result, err := ConvertLocations(cfg)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "invalid WateringPoint")
	})
}

func TestValidateLocation(t *testing.T) {
	t.Run("should return GeoJSONLocation for 2 elements", func(t *testing.T) {
		// given
		location := []float64{9.0, 48.0}

		// when
		result, err := validateLocation(location)

		// then
		require.NoError(t, err)
		assert.Equal(t, 9.0, result.Longitude)
		assert.Equal(t, 48.0, result.Latitude)
	})

	t.Run("should return error for 1 element", func(t *testing.T) {
		// given
		location := []float64{9.0}

		// when
		_, err := validateLocation(location)

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "must have exactly two elements")
	})

	t.Run("should return error for 3 elements", func(t *testing.T) {
		// given
		location := []float64{9.0, 48.0, 0.0}

		// when
		_, err := validateLocation(location)

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "must have exactly two elements")
	})

	t.Run("should return error for empty slice", func(t *testing.T) {
		// given
		location := []float64{}

		// when
		_, err := validateLocation(location)

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "must have exactly two elements")
	})
}
