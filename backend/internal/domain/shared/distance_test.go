package shared

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNewDistance(t *testing.T) {
	t.Run("valid distance", func(t *testing.T) {
		d, err := NewDistance(15.5)
		assert.NoError(t, err)
		assert.Equal(t, 15.5, d.Meters())
	})

	t.Run("zero distance", func(t *testing.T) {
		d, err := NewDistance(0)
		assert.NoError(t, err)
		assert.Equal(t, float64(0), d.Meters())
	})

	t.Run("negative distance", func(t *testing.T) {
		_, err := NewDistance(-1)
		assert.ErrorIs(t, err, ErrInvalidDistance)
	})
}

func TestMustNewDistance(t *testing.T) {
	assert.Panics(t, func() { MustNewDistance(-1) })
	assert.Equal(t, 42.0, MustNewDistance(42).Meters())
}
