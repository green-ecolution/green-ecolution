package shared

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNewSensorID(t *testing.T) {
	t.Run("valid ID", func(t *testing.T) {
		s, err := NewSensorID("sensor-001")
		assert.NoError(t, err)
		assert.Equal(t, "sensor-001", s.String())
	})

	t.Run("empty ID", func(t *testing.T) {
		_, err := NewSensorID("")
		assert.ErrorIs(t, err, ErrInvalidSensorID)
	})
}

func TestSensorID_ZeroValue(t *testing.T) {
	var s SensorID
	assert.Equal(t, "", s.String())
}

func TestMustNewSensorID(t *testing.T) {
	assert.Panics(t, func() { MustNewSensorID("") })
	assert.Equal(t, "abc", MustNewSensorID("abc").String())
}
