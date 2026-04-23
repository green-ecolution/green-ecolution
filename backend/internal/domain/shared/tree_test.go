package shared

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func testTree(plantingYearOffset int32) *Tree {
	py := MustNewPlantingYear(int32(time.Now().Year()) - plantingYearOffset)
	return &Tree{
		ID:             1,
		PlantingYear:   py,
		Coordinate:     MustNewCoordinate(54.8, 9.4),
		WateringStatus: WateringStatusUnknown,
	}
}

func watermarks(w30, w60, w90 int) []Watermark {
	return []Watermark{
		{Centibar: w30, Depth: 30},
		{Centibar: w60, Depth: 60},
		{Centibar: w90, Depth: 90},
	}
}

func TestTree_CalculateWateringStatus(t *testing.T) {
	t.Run("year 0-1: all green", func(t *testing.T) {
		tree := testTree(0)
		status, err := tree.CalculateWateringStatus(watermarks(10, 10, 10))
		assert.NoError(t, err)
		assert.Equal(t, WateringStatusGood, status)
	})

	t.Run("year 0-1: worst is moderate", func(t *testing.T) {
		tree := testTree(1)
		status, err := tree.CalculateWateringStatus(watermarks(10, 30, 10))
		assert.NoError(t, err)
		assert.Equal(t, WateringStatusModerate, status)
	})

	t.Run("year 0-1: worst is bad", func(t *testing.T) {
		tree := testTree(0)
		status, err := tree.CalculateWateringStatus(watermarks(10, 10, 50))
		assert.NoError(t, err)
		assert.Equal(t, WateringStatusBad, status)
	})

	t.Run("year 2: depth 30 uses higher thresholds", func(t *testing.T) {
		tree := testTree(2)
		// 30cm: <62 = green, 62-80 = moderate, >80 = bad
		status, err := tree.CalculateWateringStatus(watermarks(50, 10, 10))
		assert.NoError(t, err)
		assert.Equal(t, WateringStatusGood, status)

		status, err = tree.CalculateWateringStatus(watermarks(70, 10, 10))
		assert.NoError(t, err)
		assert.Equal(t, WateringStatusModerate, status)
	})

	t.Run("year 3: no moderate zone", func(t *testing.T) {
		tree := testTree(3)
		// depths 60/90: <80 = green, >=80 = bad (no moderate)
		status, err := tree.CalculateWateringStatus(watermarks(100, 50, 50))
		assert.NoError(t, err)
		assert.Equal(t, WateringStatusGood, status)

		status, err = tree.CalculateWateringStatus(watermarks(100, 90, 50))
		assert.NoError(t, err)
		assert.Equal(t, WateringStatusBad, status)
	})

	t.Run("tree beyond monitoring period returns error", func(t *testing.T) {
		tree := testTree(4)
		_, err := tree.CalculateWateringStatus(watermarks(10, 10, 10))
		assert.ErrorIs(t, err, ErrTreeBeyondMonitoring)
	})

	t.Run("malformed watermarks returns error", func(t *testing.T) {
		tree := testTree(0)
		_, err := tree.CalculateWateringStatus([]Watermark{{Centibar: 10, Depth: 30}})
		assert.ErrorIs(t, err, ErrSensorDataMalformed)
	})

	t.Run("unsorted watermarks are handled correctly", func(t *testing.T) {
		tree := testTree(0)
		wm := []Watermark{
			{Centibar: 50, Depth: 90},
			{Centibar: 10, Depth: 30},
			{Centibar: 10, Depth: 60},
		}
		status, err := tree.CalculateWateringStatus(wm)
		assert.NoError(t, err)
		assert.Equal(t, WateringStatusBad, status)
	})
}

func TestTree_AssignSensor(t *testing.T) {
	tree := testTree(0)
	sensor := &Sensor{ID: MustNewSensorID("s-1")}

	tree.AssignSensor(sensor)
	assert.Equal(t, sensor, tree.Sensor)
}

func TestTree_RemoveSensor(t *testing.T) {
	tree := testTree(0)
	tree.Sensor = &Sensor{ID: MustNewSensorID("s-1")}
	tree.WateringStatus = WateringStatusGood

	tree.RemoveSensor()
	assert.Nil(t, tree.Sensor)
	assert.Equal(t, WateringStatusUnknown, tree.WateringStatus)
}

func TestTree_IsWateringStatusExpired(t *testing.T) {
	cutoff := time.Now().Add(-24 * time.Hour)

	t.Run("expired: just watered and older than cutoff", func(t *testing.T) {
		watered := time.Now().Add(-25 * time.Hour)
		tree := &Tree{WateringStatus: WateringStatusJustWatered, LastWatered: &watered}
		assert.True(t, tree.IsWateringStatusExpired(cutoff))
	})

	t.Run("not expired: just watered but recent", func(t *testing.T) {
		watered := time.Now().Add(-1 * time.Hour)
		tree := &Tree{WateringStatus: WateringStatusJustWatered, LastWatered: &watered}
		assert.False(t, tree.IsWateringStatusExpired(cutoff))
	})

	t.Run("not expired: different status", func(t *testing.T) {
		watered := time.Now().Add(-25 * time.Hour)
		tree := &Tree{WateringStatus: WateringStatusGood, LastWatered: &watered}
		assert.False(t, tree.IsWateringStatusExpired(cutoff))
	})

	t.Run("not expired: nil last watered", func(t *testing.T) {
		tree := &Tree{WateringStatus: WateringStatusJustWatered}
		assert.False(t, tree.IsWateringStatusExpired(cutoff))
	})
}

func TestTree_RefreshWateringStatus(t *testing.T) {
	t.Run("no sensor returns error", func(t *testing.T) {
		tree := testTree(0)
		_, _, err := tree.RefreshWateringStatus()
		assert.ErrorIs(t, err, ErrNoSensorData)
	})

	t.Run("sensor without data returns error", func(t *testing.T) {
		tree := testTree(0)
		tree.Sensor = &Sensor{ID: MustNewSensorID("s-1")}
		_, _, err := tree.RefreshWateringStatus()
		assert.ErrorIs(t, err, ErrNoSensorData)
	})

	t.Run("recalculates from sensor data", func(t *testing.T) {
		tree := testTree(0)
		tree.WateringStatus = WateringStatusUnknown
		tree.Sensor = &Sensor{
			ID: MustNewSensorID("s-1"),
			LatestData: &SensorData{
				Data: &MqttPayload{
					Watermarks: watermarks(10, 10, 10),
				},
			},
		}

		status, changed, err := tree.RefreshWateringStatus()
		assert.NoError(t, err)
		assert.True(t, changed)
		assert.Equal(t, WateringStatusGood, status)
	})
}
