package mapper_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/mapper"
)

func TestSensorMapper_FromSql(t *testing.T) {
	sensorMapper := &mapper.InternalSensorRepoMapperImpl{}

	t.Run("should convert from sql to entity", func(t *testing.T) {
		// given
		src := allTestSensors[0]

		// when
		got, err := sensorMapper.FromSql(src)

		// then
		assert.NotNil(t, got)
		assert.NoError(t, err)
		assert.Equal(t, src.ID, got.ID.String())
		assert.Equal(t, src.CreatedAt, got.CreatedAt)
		assert.Equal(t, src.UpdatedAt, got.UpdatedAt)
		assert.Equal(t, src.Status, sqlc.SensorStatus(got.Status))
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src *sqlc.Sensor = nil

		// when
		got, err := sensorMapper.FromSql(src)

		// then
		assert.Nil(t, got)
		assert.NoError(t, err)
	})
}

func TestSensorMapper_FromSqlList(t *testing.T) {
	sensorMapper := &mapper.InternalSensorRepoMapperImpl{}

	t.Run("should convert from sql slice to entity slice", func(t *testing.T) {
		// given
		src := allTestSensors

		// when
		got, err := sensorMapper.FromSqlList(src)

		// then
		assert.NotNil(t, got)
		assert.NoError(t, err)
		assert.Len(t, got, 2)

		for i, src := range src {
			assert.NotNil(t, got)
			assert.Equal(t, src.ID, got[i].ID.String())
			assert.Equal(t, src.CreatedAt, got[i].CreatedAt)
			assert.Equal(t, src.UpdatedAt, got[i].UpdatedAt)
			assert.Equal(t, src.Status, sqlc.SensorStatus(got[i].Status))
		}
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src []*sqlc.Sensor = nil

		// when
		got, err := sensorMapper.FromSqlList(src)

		// then
		assert.Nil(t, got)
		assert.NoError(t, err)
	})
}

var allTestSensors = []*sqlc.Sensor{
	{
		ID:        "sensor-1",
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
		Status:    sqlc.SensorStatusOffline,
	},
	{
		ID:        "sensor-1",
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
		Status:    sqlc.SensorStatusOnline,
	},
}
