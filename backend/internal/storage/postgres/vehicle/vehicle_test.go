package vehicle

import (
	"context"
	"fmt"
	"os"
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/mapper/generated"
	store "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/testutils"
	"github.com/stretchr/testify/assert"
)

type vehicleFields struct {
	store          *store.Store
	VehicleMappers VehicleRepositoryMappers
}

var (
	defaultFields vehicleFields
	suite         *testutils.PostgresTestSuite
)

func defaultVehicleMappers() VehicleRepositoryMappers {
	return NewVehicleRepositoryMappers(&generated.InternalVehicleRepoMapperImpl{})
}

func TestMain(m *testing.M) {
	code := 1
	ctx := context.Background()
	defer func() { os.Exit(code) }()
	suite = testutils.SetupPostgresTestSuite(ctx)
	defaultFields = vehicleFields{
		store:          suite.Store,
		VehicleMappers: defaultVehicleMappers(),
	}
	defer suite.Terminate(ctx)

	code = m.Run()
}

func TestVehicleRepository_Delete(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/vehicle")

	t.Run("should delete vehicle", func(t *testing.T) {
		// given
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		// when
		err := r.Delete(context.Background(), 1)

		// then
		assert.NoError(t, err)
	})

	t.Run("should return error when vehicle not found", func(t *testing.T) {
		// given
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		// when
		err := r.Delete(context.Background(), 99)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when tree cluster with negative id", func(t *testing.T) {
		// given
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())

		// when
		err := r.Delete(context.Background(), -1)
		fmt.Print(err)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when vehicle is canceled", func(t *testing.T) {
		// given
		r := NewVehicleRepository(suite.Store, defaultVehicleMappers())
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		err := r.Delete(ctx, 1)

		// then
		assert.Error(t, err)
	})
}
