package sensor

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

func TestSensorRepository_Update(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")

	t.Run("should update sensor successfully", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())
		newCoordinate := shared.MustNewCoordinate(54.82078826498143, 9.489684366114483)
		newLatestData := &shared.SensorData{
			ID:        1,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
			Data:      TestMqttPayload,
		}

		got, err := r.Update(context.Background(), shared.MustNewSensorID("sensor-1"), func(sensor *shared.Sensor, _ shared.SensorRepository) (bool, error) {
			sensor.Status = shared.SensorStatusOffline
			sensor.Coordinate = newCoordinate
			sensor.LatestData = newLatestData
			return true, nil
		})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, shared.SensorStatusOffline, got.Status)
		assert.Equal(t, newCoordinate.Latitude(), got.Coordinate.Latitude())
		assert.Equal(t, newCoordinate.Longitude(), got.Coordinate.Longitude())

		assert.NotZero(t, got.LatestData.UpdatedAt)
		assert.NotZero(t, got.LatestData.CreatedAt)
		assert.Equal(t, TestMqttPayload, got.LatestData.Data)
	})

	t.Run("should return error when update sensor with empty name", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.Update(context.Background(), shared.MustNewSensorID("sensor-1"), func(sensor *shared.Sensor, _ shared.SensorRepository) (bool, error) {
			sensor.Status = ""
			return true, nil
		})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when update sensor with empty id", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.Update(context.Background(), shared.SensorID{}, func(sensor *shared.Sensor, _ shared.SensorRepository) (bool, error) {
			return true, nil
		})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when sensor not found", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.Update(context.Background(), shared.MustNewSensorID("notFoundID"), func(sensor *shared.Sensor, _ shared.SensorRepository) (bool, error) {
			return true, nil
		})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.Update(ctx, shared.MustNewSensorID("sensor-1"), func(sensor *shared.Sensor, _ shared.SensorRepository) (bool, error) {
			sensor.Status = shared.SensorStatusOffline
			return true, nil
		})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}
