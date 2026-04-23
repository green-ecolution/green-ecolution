package entities

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNewCoordinate(t *testing.T) {
	t.Run("valid coordinate", func(t *testing.T) {
		c, err := NewCoordinate(54.801539, 9.446741)
		assert.NoError(t, err)
		assert.Equal(t, 54.801539, c.Latitude())
		assert.Equal(t, 9.446741, c.Longitude())
	})

	t.Run("zero coordinate", func(t *testing.T) {
		c, err := NewCoordinate(0, 0)
		assert.NoError(t, err)
		assert.Equal(t, float64(0), c.Latitude())
		assert.Equal(t, float64(0), c.Longitude())
	})

	t.Run("boundary values", func(t *testing.T) {
		c, err := NewCoordinate(90, 180)
		assert.NoError(t, err)
		assert.Equal(t, float64(90), c.Latitude())
		assert.Equal(t, float64(180), c.Longitude())

		c, err = NewCoordinate(-90, -180)
		assert.NoError(t, err)
		assert.Equal(t, float64(-90), c.Latitude())
		assert.Equal(t, float64(-180), c.Longitude())
	})

	t.Run("invalid latitude too high", func(t *testing.T) {
		_, err := NewCoordinate(91, 0)
		assert.ErrorIs(t, err, ErrInvalidLatitude)
	})

	t.Run("invalid latitude too low", func(t *testing.T) {
		_, err := NewCoordinate(-91, 0)
		assert.ErrorIs(t, err, ErrInvalidLatitude)
	})

	t.Run("invalid longitude too high", func(t *testing.T) {
		_, err := NewCoordinate(0, 181)
		assert.ErrorIs(t, err, ErrInvalidLongitude)
	})

	t.Run("invalid longitude too low", func(t *testing.T) {
		_, err := NewCoordinate(0, -181)
		assert.ErrorIs(t, err, ErrInvalidLongitude)
	})
}

func TestNewCoordinateFromOptional(t *testing.T) {
	t.Run("both nil", func(t *testing.T) {
		c, err := NewCoordinateFromOptional(nil, nil)
		assert.NoError(t, err)
		assert.Nil(t, c)
	})

	t.Run("lat nil", func(t *testing.T) {
		lng := 9.0
		c, err := NewCoordinateFromOptional(nil, &lng)
		assert.NoError(t, err)
		assert.Nil(t, c)
	})

	t.Run("valid values", func(t *testing.T) {
		lat, lng := 54.8, 9.4
		c, err := NewCoordinateFromOptional(&lat, &lng)
		assert.NoError(t, err)
		assert.NotNil(t, c)
		assert.Equal(t, 54.8, c.Latitude())
		assert.Equal(t, 9.4, c.Longitude())
	})

	t.Run("invalid values", func(t *testing.T) {
		lat, lng := 91.0, 9.4
		_, err := NewCoordinateFromOptional(&lat, &lng)
		assert.ErrorIs(t, err, ErrInvalidLatitude)
	})
}

func TestMustNewCoordinate(t *testing.T) {
	t.Run("panics on invalid", func(t *testing.T) {
		assert.Panics(t, func() { MustNewCoordinate(91, 0) })
	})

	t.Run("returns on valid", func(t *testing.T) {
		c := MustNewCoordinate(54.8, 9.4)
		assert.Equal(t, 54.8, c.Latitude())
	})
}
