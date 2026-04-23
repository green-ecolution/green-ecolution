package shared

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNewWaterCapacity(t *testing.T) {
	t.Run("valid capacity", func(t *testing.T) {
		w, err := NewWaterCapacity(1000.5)
		assert.NoError(t, err)
		assert.Equal(t, 1000.5, w.Liters())
	})

	t.Run("zero capacity", func(t *testing.T) {
		w, err := NewWaterCapacity(0)
		assert.NoError(t, err)
		assert.Equal(t, float64(0), w.Liters())
	})

	t.Run("negative capacity", func(t *testing.T) {
		_, err := NewWaterCapacity(-1)
		assert.ErrorIs(t, err, ErrInvalidWaterCapacity)
	})
}

func TestWaterCapacity_Add(t *testing.T) {
	a := MustNewWaterCapacity(100)
	b := MustNewWaterCapacity(50)
	result := a.Add(b)
	assert.Equal(t, 150.0, result.Liters())
}

func TestMustNewWaterCapacity(t *testing.T) {
	assert.Panics(t, func() { MustNewWaterCapacity(-1) })
	assert.Equal(t, 42.0, MustNewWaterCapacity(42).Liters())
}
