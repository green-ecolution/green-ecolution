package sensor

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

func TestSensorRepository_GetAll(t *testing.T) {
	t.Run("should return all sensors without limitation", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, len(TestSensorList), len(got))
		assert.Equal(t, totalCount, int64(len(TestSensorList)))

		for i, sensor := range got {
			assert.Equal(t, TestSensorList[i].ID, sensor.ID)
			assert.Equal(t, TestSensorList[i].Status, sensor.Status)
			assert.Equal(t, TestSensorList[i].Coordinate.Latitude(), sensor.Coordinate.Latitude())
			assert.Equal(t, TestSensorList[i].Coordinate.Longitude(), sensor.Coordinate.Longitude())
			assert.NotZero(t, sensor.CreatedAt)
			assert.NotZero(t, sensor.UpdatedAt)

			// assert latest data
			if TestSensorList[i].LatestData != nil {
				assert.NotZero(t, sensor.LatestData.UpdatedAt)
				assert.NotZero(t, sensor.LatestData.CreatedAt)
				assert.Equal(t, TestSensorList[i].LatestData.Data, sensor.LatestData.Data)
			}
		}
	})

	t.Run("should return all sensors without limitation with provider", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		exptectedSensor := TestSensorList[len(TestSensorList)-1]

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{Provider: "test-provider"})

		// then
		assert.NoError(t, err)
		assert.Equal(t, 1, len(got))
		assert.Equal(t, totalCount, int64(1))

		for _, sensor := range got {
			assert.Equal(t, exptectedSensor.ID, sensor.ID)
			assert.Equal(t, exptectedSensor.Status, sensor.Status)
			assert.Equal(t, exptectedSensor.Coordinate.Latitude(), sensor.Coordinate.Latitude())
			assert.Equal(t, exptectedSensor.Coordinate.Longitude(), sensor.Coordinate.Longitude())
			assert.Equal(t, exptectedSensor.AdditionalInfo, sensor.AdditionalInfo)
			assert.Equal(t, exptectedSensor.Provider, sensor.Provider)
			assert.NotZero(t, sensor.CreatedAt)
			assert.NotZero(t, sensor.UpdatedAt)
		}
	})

	t.Run("should return all sensors limited by 2 and with an offset of 2", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.NoError(t, err)
		assert.NotEmpty(t, got)
		assert.Equal(t, totalCount, int64(len(TestSensorList)))

		sensors := TestSensorList[2:4]

		for i, sensor := range got {
			assert.Equal(t, sensors[i].ID, sensor.ID)
		}
	})

	t.Run("should return error on invalid page value", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		ctx := context.WithValue(context.Background(), "page", int32(0))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error on invalid limit value", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(0))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return empty slice when db is empty", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, _, err := r.GetAll(ctx, entities.Query{})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestSensorRepository_GetCount(t *testing.T) {
	t.Run("should return count of all sensors in db", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		totalCount, err := r.GetCount(context.Background(), entities.Query{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, int64(len(TestSensorList)), totalCount)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		totalCount, err := r.GetCount(ctx, entities.Query{})

		// then
		assert.Error(t, err)
		assert.Equal(t, int64(0), totalCount)
	})
}

func TestSensorRepository_GetAllDataById(t *testing.T) {
	t.Run("should return all sensor data", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.GetAllDataByID(context.Background(), entities.MustNewSensorID("sensor-1"))

		// then
		assert.NoError(t, err)
		assert.Equal(t, len(TestSensorData), len(got))

		for i, sensor := range got {
			assert.Equal(t, TestSensorData[i].SensorID, sensor.SensorID)
			assert.Equal(t, TestSensorData[i].Data.Battery, sensor.Data.Battery)
			assert.Equal(t, TestSensorData[i].Data.Device, sensor.Data.Device)
			assert.Equal(t, TestSensorData[i].Data.Humidity, sensor.Data.Humidity)
			assert.Equal(t, TestSensorData[i].Data.Temperature, sensor.Data.Temperature)
			assert.NotZero(t, sensor.CreatedAt)
			assert.NotZero(t, sensor.UpdatedAt)
		}
	})

	t.Run("should return error when no data is found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.GetAllDataByID(context.Background(), entities.MustNewSensorID("sensor-999"))

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when id is invalid", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.GetAllDataByID(context.Background(), entities.SensorID{})

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
		got, err := r.GetAllDataByID(ctx, entities.MustNewSensorID("sensor-1"))

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestSensorRepository_GetByID(t *testing.T) {
	t.Run("should return sensor by id", func(t *testing.T) {
		// given
		ctx := context.Background()
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.GetByID(ctx, entities.MustNewSensorID("sensor-1"))

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestSensorList[0].ID, got.ID)
		assert.Equal(t, TestSensorList[0].Status, got.Status)
		assert.Equal(t, TestSensorList[0].Coordinate.Latitude(), got.Coordinate.Latitude())
		assert.Equal(t, TestSensorList[0].Coordinate.Longitude(), got.Coordinate.Longitude())
		assert.NotZero(t, got.CreatedAt)
		assert.NotZero(t, got.UpdatedAt)

		// assert latest data
		assert.NotZero(t, got.LatestData.UpdatedAt)
		assert.NotZero(t, got.LatestData.CreatedAt)
		assert.Equal(t, TestSensorList[0].LatestData.Data, got.LatestData.Data)
	})

	t.Run("should return error when sensor not found", func(t *testing.T) {
		// given
		ctx := context.Background()
		suite.ResetDB(t)
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.GetByID(ctx, entities.MustNewSensorID("sensor-1"))

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when sensor id is empty", func(t *testing.T) {
		// given
		ctx := context.Background()
		suite.ResetDB(t)
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.GetByID(ctx, entities.SensorID{})

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
		got, err := r.GetByID(ctx, entities.MustNewSensorID("sensor-1"))

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestSensorRepository_GetLastSensorDataByID(t *testing.T) {
	t.Run("should return last sensor data for valid id", func(t *testing.T) {
		// given
		ctx := context.Background()
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		data, err := r.GetLatestSensorDataBySensorID(ctx, entities.MustNewSensorID("sensor-1"))

		// then
		assert.NoError(t, err)
		assert.NotEmpty(t, data)
		assert.Equal(t, int32(1), data.ID)
		assert.NotZero(t, data.CreatedAt)
		assert.NotZero(t, data.UpdatedAt)
	})

	t.Run("should return error when no data found", func(t *testing.T) {
		// given
		ctx := context.Background()
		suite.ResetDB(t)
		r := NewSensorRepository(suite.Store, defaultSensorMappers())

		// when
		got, err := r.GetLatestSensorDataBySensorID(ctx, entities.MustNewSensorID("notFoundID"))

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
		// assert.EqualError(t, err, error.Error(entities.ErrEntityNotFound))
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewSensorRepository(suite.Store, defaultSensorMappers())
		suite.ResetDB(t)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.GetLatestSensorDataBySensorID(ctx, entities.MustNewSensorID("sensor-1"))

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

var TestSensorList = []*entities.Sensor{
	{
		ID:         entities.MustNewSensorID("sensor-1"),
		CreatedAt:  time.Now(),
		UpdatedAt:  time.Now(),
		Coordinate: entities.MustNewCoordinate(54.82124518093376, 9.485702120628517),
		Status:     entities.SensorStatusOnline,
		LatestData: &entities.SensorData{
			ID:        1,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
			Data:      TestMqttPayload,
		},
	},
	{
		ID:         entities.MustNewSensorID("sensor-2"),
		CreatedAt:  time.Now(),
		UpdatedAt:  time.Now(),
		Coordinate: entities.MustNewCoordinate(54.78780993841013, 9.444052105200551),
		Status:     entities.SensorStatusOffline,
	},
	{
		ID:         entities.MustNewSensorID("sensor-3"),
		CreatedAt:  time.Now(),
		UpdatedAt:  time.Now(),
		Coordinate: entities.MustNewCoordinate(54.77933725347423, 9.426465409018832),
		Status:     entities.SensorStatusUnknown,
	},
	{
		ID:         entities.MustNewSensorID("sensor-4"),
		CreatedAt:  time.Now(),
		UpdatedAt:  time.Now(),
		Coordinate: entities.MustNewCoordinate(54.82078826498143, 9.489684366114483),
		Status:     entities.SensorStatusOnline,
	},
	{
		ID:         entities.MustNewSensorID("sensor-provider"),
		CreatedAt:  time.Now(),
		UpdatedAt:  time.Now(),
		Coordinate: entities.MustNewCoordinate(54.82078826498143, 9.489684366114483),
		Status:     entities.SensorStatusOnline,
		Provider:   "test-provider",
		AdditionalInfo: map[string]interface{}{
			"foo": "bar",
		},
	},
}

var TestMqttPayload = &entities.MqttPayload{
	Device:      "sensor-123",
	Battery:     34.0,
	Humidity:    50,
	Temperature: 20,
	Watermarks: []entities.Watermark{
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

var TestSensorData = []*entities.SensorData{
	{
		ID:        1,
		SensorID:  entities.MustNewSensorID("sensor-1"),
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
		Data:      TestMqttPayload,
	},
	{
		ID:        2,
		SensorID:  entities.MustNewSensorID("sensor-1"),
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
		Data: &entities.MqttPayload{
			Device:      "sensor-123",
			Battery:     32.0,
			Humidity:    40,
			Temperature: 10,
			Watermarks: []entities.Watermark{
				{
					Resistance: 20,
					Centibar:   10,
					Depth:      30,
				},
				{
					Resistance: 20,
					Centibar:   10,
					Depth:      60,
				},
				{
					Resistance: 20,
					Centibar:   10,
					Depth:      90,
				},
			},
		},
	},
}
