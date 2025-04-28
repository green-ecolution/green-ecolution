package mapper_test

import (
	"testing"
	"time"

	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/backend/internal/storage/postgres/mapper/generated"
	"github.com/jackc/pgx/v5/pgtype"
	"github.com/stretchr/testify/assert"
)

func TestSensorMapper_FromSql(t *testing.T) {
	sensorMapper := &generated.InternalSensorRepoMapperImpl{}

	t.Run("should convert from sql to entity", func(t *testing.T) {
		// given
		src := allTestSensors[0]

		// when
		got, err := sensorMapper.FromSql(src)

		// then
		assert.NotNil(t, got)
		assert.NoError(t, err)
		assert.Equal(t, src.ID, got.ID)
		assert.Equal(t, src.CreatedAt.Time, got.CreatedAt)
		assert.Equal(t, src.UpdatedAt.Time, got.UpdatedAt)
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
	sensorMapper := &generated.InternalSensorRepoMapperImpl{}

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
			assert.Equal(t, src.ID, got[i].ID)
			assert.Equal(t, src.CreatedAt.Time, got[i].CreatedAt)
			assert.Equal(t, src.UpdatedAt.Time, got[i].UpdatedAt)
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
		CreatedAt: pgtype.Timestamp{Time: time.Now()},
		UpdatedAt: pgtype.Timestamp{Time: time.Now()},
		Status:    sqlc.SensorStatusOffline,
	},
	{
		ID:        "sensor-1",
		CreatedAt: pgtype.Timestamp{Time: time.Now()},
		UpdatedAt: pgtype.Timestamp{Time: time.Now()},
		Status:    sqlc.SensorStatusOnline,
	},
}
