package sensor

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	sensorDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

func TestSensorRepository_Create(t *testing.T) {
	suite.ResetDB(t)

	t.Run("should create sensor", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.Create(context.Background(), &sensorDomain.Sensor{
			ID:         input.ID,
			Coordinate: input.Coordinate,
			Status:     input.Status,
			LatestData: input.LatestData,
		})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, input.Coordinate.Latitude(), got.Coordinate.Latitude())
		assert.Equal(t, input.Coordinate.Longitude(), got.Coordinate.Longitude())
		assert.Equal(t, input.Status, got.Status)
		assert.NotZero(t, got.ID)

		// assert latest data
		assert.NotZero(t, got.LatestData.UpdatedAt)
		assert.NotZero(t, got.LatestData.CreatedAt)
		assert.Equal(t, input.LatestData.Data, got.LatestData.Data)
	})

	t.Run("should create sensor with empty data and unknown status", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.Create(context.Background(), &sensorDomain.Sensor{
			ID:         sensorDomain.MustNewSensorID("sensor-124"),
			Status:     sensorDomain.SensorStatusUnknown,
			Coordinate: input.Coordinate,
		})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, sensorDomain.SensorStatusUnknown, got.Status)
		assert.Nil(t, got.LatestData)
		assert.Equal(t, input.Coordinate.Latitude(), got.Coordinate.Latitude())
		assert.Equal(t, input.Coordinate.Longitude(), got.Coordinate.Longitude())
		assert.NotZero(t, got.ID)

		// assert latest data
		assert.Nil(t, got.LatestData)
	})

	t.Run("should return error if sensor id is invalid", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.Create(context.Background(), &sensorDomain.Sensor{
			Coordinate: input.Coordinate,
		})

		// then
		assert.Error(t, err)
		assert.Equal(t, err.Error(), "sensor id cannot be empty")
		assert.Nil(t, got)
	})

	t.Run("should return error if sensor with same id already exists", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.Create(context.Background(), &sensorDomain.Sensor{
			ID:         input.ID,
			Coordinate: input.Coordinate,
			Status:     input.Status,
			LatestData: input.LatestData,
		})

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "sensor with same ID already exists")
		assert.Nil(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.Create(ctx, &sensorDomain.Sensor{
			ID: sensorDomain.MustNewSensorID("sensor-5"),
		})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestSensorRepository_InsertSensorData(t *testing.T) {
	suite.ResetDB(t)

	t.Run("should insert sensor data successfully", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		_, err := r.Create(context.Background(), &sensorDomain.Sensor{
			ID:         input.ID,
			Coordinate: input.Coordinate,
			Status:     input.Status,
		})

		assert.NoError(t, err)

		// when
		err = r.InsertSensorData(context.Background(), input.LatestData, input.ID)

		// then
		assert.NoError(t, err)
	})

	t.Run("should return error when data is empty", func(t *testing.T) {
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		_, err := r.Create(context.Background(), &sensorDomain.Sensor{
			ID:         sensorDomain.MustNewSensorID("sensor-124"),
			Coordinate: input.Coordinate,
			Status:     input.Status,
		})

		assert.NoError(t, err)

		// when
		err = r.InsertSensorData(context.Background(), &sensorDomain.SensorData{}, input.ID)

		// then
		assert.Error(t, err)
		assert.Equal(t, err.Error(), "latest data cannot be empty")
	})

	t.Run("should return error when data is nil", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		err := r.InsertSensorData(context.Background(), nil, sensorDomain.MustNewSensorID("sensor-1"))

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when sensor id is invalid", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		err := r.InsertSensorData(context.Background(), input.LatestData, sensorDomain.SensorID{})

		// then
		assert.Error(t, err)
	})
}

var inputPayload = &sensorDomain.MqttPayload{
	Device:      "sensor-123",
	Battery:     34.0,
	Humidity:    50,
	Temperature: 20,
	Watermarks: []sensorDomain.Watermark{
		{
			Resistance: 23,
			Centibar:   38,
			Depth:      30,
		},
		{
			Resistance: 23,
			Centibar:   38,
			Depth:      60,
		},
		{
			Resistance: 23,
			Centibar:   38,
			Depth:      90,
		},
	},
}

var input = &sensorDomain.SensorCreate{
	ID:     sensorDomain.MustNewSensorID("sensor-123"),
	Status: sensorDomain.SensorStatusOnline,
	LatestData: &sensorDomain.SensorData{
		ID:        1,
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
		Data:      inputPayload,
	},
	Coordinate: shared.MustNewCoordinate(9.446741, 54.801539),
}
