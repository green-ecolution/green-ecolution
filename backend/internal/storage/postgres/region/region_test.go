package region

import (
	"context"
	"os"
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/mapper/generated"
	store "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/testutils"
	"github.com/stretchr/testify/assert"
)

type regionFields struct {
	store         *store.Store
	RegionMappers RegionMappers
}

var (
	defaultFields regionFields
	suite         *testutils.PostgresTestSuite
)

func defaultRegionMappers() RegionMappers {
	return NewRegionMappers(&generated.InternalRegionRepoMapperImpl{})
}

func TestMain(m *testing.M) {
	code := 1
	ctx := context.Background()
	defer func() { os.Exit(code) }()
	suite = testutils.SetupPostgresTestSuite(ctx)
	defaultFields = regionFields{
		store:         suite.Store,
		RegionMappers: defaultRegionMappers(),
	}
	defer suite.Terminate(ctx)

	code = m.Run()
}

func TestRegionRepository_Delete(t *testing.T) {
	t.Run("should delete region", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/region")
		r := NewRegionRepository(defaultFields.store, defaultFields.RegionMappers)

		// when
		err := r.Delete(context.Background(), 1)

		// then
		assert.NoError(t, err)
	})

	t.Run("should return error when region not found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewRegionRepository(defaultFields.store, defaultFields.RegionMappers)

		// when
		err := r.Delete(context.Background(), 99)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewRegionRepository(defaultFields.store, defaultFields.RegionMappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		err := r.Delete(ctx, 1)

		// then
		assert.Error(t, err)
	})
}
