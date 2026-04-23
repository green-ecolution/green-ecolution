package entities

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestTreeCluster_YoungestTree(t *testing.T) {
	t.Run("returns youngest tree", func(t *testing.T) {
		tc := &TreeCluster{
			Trees: []*Tree{
				{ID: 1, PlantingYear: MustNewPlantingYear(2020)},
				{ID: 2, PlantingYear: MustNewPlantingYear(2023)},
				{ID: 3, PlantingYear: MustNewPlantingYear(2021)},
			},
		}
		youngest := tc.YoungestTree()
		assert.NotNil(t, youngest)
		assert.Equal(t, int32(1), youngest.ID)
	})

	t.Run("returns nil for empty trees", func(t *testing.T) {
		tc := &TreeCluster{Trees: []*Tree{}}
		assert.Nil(t, tc.YoungestTree())
	})

	t.Run("returns nil for nil trees", func(t *testing.T) {
		tc := &TreeCluster{}
		assert.Nil(t, tc.YoungestTree())
	})
}

func TestTreeCluster_AverageWatermarks(t *testing.T) {
	t.Run("averages watermarks across sensors", func(t *testing.T) {
		tc := &TreeCluster{}
		sensorData := []*SensorData{
			{Data: &MqttPayload{Watermarks: []Watermark{
				{Centibar: 20, Depth: 30}, {Centibar: 30, Depth: 60}, {Centibar: 40, Depth: 90},
			}}},
			{Data: &MqttPayload{Watermarks: []Watermark{
				{Centibar: 40, Depth: 30}, {Centibar: 50, Depth: 60}, {Centibar: 60, Depth: 90},
			}}},
		}

		wm, err := tc.AverageWatermarks(sensorData)
		assert.NoError(t, err)
		assert.Len(t, wm, 3)
		assert.Equal(t, 30, wm[0].Centibar) // (20+40)/2
		assert.Equal(t, 40, wm[1].Centibar) // (30+50)/2
		assert.Equal(t, 50, wm[2].Centibar) // (40+60)/2
	})

	t.Run("returns error for malformed watermarks", func(t *testing.T) {
		tc := &TreeCluster{}
		sensorData := []*SensorData{
			{Data: &MqttPayload{Watermarks: []Watermark{{Centibar: 10, Depth: 30}}}},
		}

		_, err := tc.AverageWatermarks(sensorData)
		assert.ErrorIs(t, err, ErrSensorDataMalformed)
	})
}

func TestTreeCluster_CalculateWateringStatus(t *testing.T) {
	t.Run("calculates status from sensor data", func(t *testing.T) {
		tc := &TreeCluster{
			Trees: []*Tree{
				{ID: 1, PlantingYear: MustNewPlantingYear(int32(time.Now().Year()))},
			},
		}
		sensorData := []*SensorData{
			{Data: &MqttPayload{Watermarks: []Watermark{
				{Centibar: 10, Depth: 30}, {Centibar: 10, Depth: 60}, {Centibar: 10, Depth: 90},
			}}},
		}

		status, err := tc.CalculateWateringStatus(sensorData)
		assert.NoError(t, err)
		assert.Equal(t, WateringStatusGood, status)
	})

	t.Run("returns unknown for empty sensor data", func(t *testing.T) {
		tc := &TreeCluster{Trees: []*Tree{
			{ID: 1, PlantingYear: MustNewPlantingYear(2023)},
		}}

		status, err := tc.CalculateWateringStatus(nil)
		assert.NoError(t, err)
		assert.Equal(t, WateringStatusUnknown, status)
	})

	t.Run("returns unknown for no trees", func(t *testing.T) {
		tc := &TreeCluster{Trees: []*Tree{}}
		sensorData := []*SensorData{
			{Data: &MqttPayload{Watermarks: []Watermark{
				{Centibar: 10, Depth: 30}, {Centibar: 10, Depth: 60}, {Centibar: 10, Depth: 90},
			}}},
		}

		status, err := tc.CalculateWateringStatus(sensorData)
		assert.NoError(t, err)
		assert.Equal(t, WateringStatusUnknown, status)
	})
}

func TestTreeCluster_NeedsPositionUpdate(t *testing.T) {
	coord1 := MustNewCoordinate(54.8, 9.4)
	coord2 := MustNewCoordinate(54.9, 9.5)
	tc1 := &TreeCluster{ID: 1}
	tc2 := &TreeCluster{ID: 2}
	sensor1 := &Sensor{ID: MustNewSensorID("s-1")}

	t.Run("needs update when coordinate changed", func(t *testing.T) {
		prev := &Tree{Coordinate: coord1, TreeCluster: tc1}
		next := &Tree{Coordinate: coord2, TreeCluster: tc1}
		assert.True(t, tc1.NeedsPositionUpdate(prev, next))
	})

	t.Run("needs update when cluster changed", func(t *testing.T) {
		prev := &Tree{Coordinate: coord1, TreeCluster: tc1}
		next := &Tree{Coordinate: coord1, TreeCluster: tc2}
		assert.True(t, tc1.NeedsPositionUpdate(prev, next))
	})

	t.Run("needs update when sensor changed", func(t *testing.T) {
		prev := &Tree{Coordinate: coord1, TreeCluster: tc1}
		next := &Tree{Coordinate: coord1, TreeCluster: tc1, Sensor: sensor1}
		assert.True(t, tc1.NeedsPositionUpdate(prev, next))
	})

	t.Run("no update needed when nothing changed", func(t *testing.T) {
		prev := &Tree{Coordinate: coord1, TreeCluster: tc1, Sensor: sensor1}
		next := &Tree{Coordinate: coord1, TreeCluster: tc1, Sensor: sensor1}
		assert.False(t, tc1.NeedsPositionUpdate(prev, next))
	})

	t.Run("needs update when cluster added", func(t *testing.T) {
		prev := &Tree{Coordinate: coord1}
		next := &Tree{Coordinate: coord1, TreeCluster: tc1}
		assert.True(t, tc1.NeedsPositionUpdate(prev, next))
	})
}
