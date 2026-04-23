package sensor

import (
	"context"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/testutils"
)

type sensorFields struct {
	store         *store.Store
	SensorMappers SensorRepositoryMappers
}

var (
	defaultFields sensorFields
	suite         *testutils.PostgresTestSuite
)

func defaultSensorMappers() SensorRepositoryMappers {
	return NewSensorRepositoryMappers(&mapper.InternalSensorRepoMapperImpl{})
}

func TestMain(m *testing.M) {
	code := 1
	ctx := context.Background()
	defer func() { os.Exit(code) }()
	suite = testutils.SetupPostgresTestSuite(ctx)
	defaultFields = sensorFields{
		store:         suite.Store,
		SensorMappers: defaultSensorMappers(),
	}
	defer suite.Terminate(ctx)

	code = m.Run()
}

func TestSensorRepository_Delete(t *testing.T) {
	t.Run("should delete sensor", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/sensor")
		r := NewSensorRepository(defaultFields.store, defaultFields.SensorMappers)

		// when
		err := r.Delete(context.Background(), shared.MustNewSensorID("sensor-1"))

		// then
		assert.NoError(t, err)
	})

	t.Run("should return error when sensor not found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewSensorRepository(defaultFields.store, defaultFields.SensorMappers)

		// when
		err := r.Delete(context.Background(), shared.MustNewSensorID("notFoundID"))

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewSensorRepository(defaultFields.store, defaultFields.SensorMappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		err := r.Delete(ctx, shared.MustNewSensorID("sensor-1"))

		// then
		assert.Error(t, err)
	})
}
