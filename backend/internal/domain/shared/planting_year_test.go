package entities

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestNewPlantingYear(t *testing.T) {
	t.Run("valid year", func(t *testing.T) {
		p, err := NewPlantingYear(2023)
		assert.NoError(t, err)
		assert.Equal(t, int32(2023), p.Year())
	})

	t.Run("current year", func(t *testing.T) {
		p, err := NewPlantingYear(int32(time.Now().Year()))
		assert.NoError(t, err)
		assert.Equal(t, int32(time.Now().Year()), p.Year())
	})

	t.Run("zero year", func(t *testing.T) {
		_, err := NewPlantingYear(0)
		assert.ErrorIs(t, err, ErrInvalidPlantingYear)
	})

	t.Run("negative year", func(t *testing.T) {
		_, err := NewPlantingYear(-1)
		assert.ErrorIs(t, err, ErrInvalidPlantingYear)
	})

	t.Run("future year", func(t *testing.T) {
		_, err := NewPlantingYear(int32(time.Now().Year() + 1))
		assert.ErrorIs(t, err, ErrInvalidPlantingYear)
	})
}

func TestPlantingYear_ZeroValue(t *testing.T) {
	var p PlantingYear
	assert.Equal(t, int32(0), p.Year())
}

func TestMustNewPlantingYear(t *testing.T) {
	assert.Panics(t, func() { MustNewPlantingYear(0) })
	assert.Equal(t, int32(2020), MustNewPlantingYear(2020).Year())
}
