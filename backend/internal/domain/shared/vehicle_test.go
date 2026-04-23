package entities

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestMergeVehicles(t *testing.T) {
	t.Run("merges transporter and trailer", func(t *testing.T) {
		transporter := &Vehicle{
			NumberPlate:   "FL-GE 1",
			Width:         2.5,
			Height:        3.0,
			Length:        6.0,
			Weight:        3500,
			WaterCapacity: MustNewWaterCapacity(100),
			Type:          VehicleTypeTransporter,
		}
		trailer := &Vehicle{
			NumberPlate:   "FL-GE 2",
			Width:         2.0,
			Height:        3.5,
			Length:        4.0,
			Weight:        1500,
			WaterCapacity: MustNewWaterCapacity(500),
			Type:          VehicleTypeTrailer,
		}

		merged := MergeVehicles(transporter, trailer)
		assert.Equal(t, 2.5, merged.Width)                    // max
		assert.Equal(t, 3.5, merged.Height)                   // max
		assert.Equal(t, 10.0, merged.Length)                  // sum
		assert.Equal(t, 5000.0, merged.Weight)                // sum
		assert.Equal(t, 600.0, merged.WaterCapacity.Liters()) // sum
		assert.Equal(t, VehicleTypeTransporter, merged.Type)
		assert.Equal(t, "FL-GE 1 - FL-GE 2", merged.NumberPlate)
	})

	t.Run("returns transporter when no trailer", func(t *testing.T) {
		transporter := &Vehicle{NumberPlate: "FL-GE 1"}
		assert.Equal(t, transporter, MergeVehicles(transporter, nil))
	})

	t.Run("returns nil when no transporter", func(t *testing.T) {
		assert.Nil(t, MergeVehicles(nil, nil))
	})
}
