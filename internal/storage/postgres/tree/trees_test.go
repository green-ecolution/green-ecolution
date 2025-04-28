package tree

import (
	"context"
	"os"
	"testing"

	"github.com/green-ecolution/backend/internal/storage/postgres/mapper/generated"
	"github.com/green-ecolution/backend/internal/storage/postgres/testutils"
	"github.com/stretchr/testify/assert"
)

var (
	mappers TreeMappers
	suite   *testutils.PostgresTestSuite
)

func TestMain(m *testing.M) {
	code := 1
	ctx := context.Background()
	defer func() { os.Exit(code) }()
	suite = testutils.SetupPostgresTestSuite(ctx)
	mappers = NewTreeRepositoryMappers(
		&generated.InternalTreeRepoMapperImpl{},
		&generated.InternalSensorRepoMapperImpl{},
		&generated.InternalTreeClusterRepoMapperImpl{},
	)
	defer suite.Terminate(ctx)
	code = m.Run()
}

func TestTreeRepository_Delete(t *testing.T) {
	t.Run("should delete tree successfully", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		treeID := int32(1)

		// when
		err := r.Delete(context.Background(), treeID)

		// then
		assert.NoError(t, err)

		deletedTree, err := r.GetByID(context.Background(), treeID)
		assert.Error(t, err)
		assert.Nil(t, deletedTree)
	})

	t.Run("should return error if tree not found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		err := r.Delete(context.Background(), 99)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error if tree ID is negative", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		err := r.Delete(context.Background(), -1)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error if tree ID is zero", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		err := r.Delete(context.Background(), 0)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		err := r.Delete(ctx, 1)

		// then
		assert.Error(t, err)
	})
}

func TestTreeRepository_UnlinkTreeClusterID(t *testing.T) {
	t.Run("should unlink tree cluster ID from a tree successfully", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		treeClusterID := int32(1)
		trees, err := suite.Store.GetTreesByTreeClusterID(context.Background(), &treeClusterID)
		assert.NoError(t, err)
		assert.NotEmpty(t, trees)

		// when
		err = r.UnlinkTreeClusterID(context.Background(), treeClusterID)

		// then
		assert.NoError(t, err)
		trees, err = suite.Store.GetTreesByTreeClusterID(context.Background(), &treeClusterID)
		assert.NoError(t, err)
		assert.Empty(t, trees)
	})

	t.Run("should return empty trees when tree cluster is not set in any tree", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		treeClusterID := int32(6)
		trees, err := suite.Store.GetTreesByTreeClusterID(context.Background(), &treeClusterID)
		assert.NoError(t, err)
		assert.Empty(t, trees)

		// when
		err = r.UnlinkTreeClusterID(context.Background(), treeClusterID)

		// then
		assert.NoError(t, err)
		trees, err = suite.Store.GetTreesByTreeClusterID(context.Background(), &treeClusterID)
		assert.NoError(t, err)
		assert.Empty(t, trees)
	})

	t.Run("should return error when tree cluster not found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		// when
		err := r.UnlinkTreeClusterID(context.Background(), 99)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when tree cluster id negative", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		// when
		err := r.UnlinkTreeClusterID(context.Background(), -1)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when tree cluster id is zero", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		// when
		err := r.UnlinkTreeClusterID(context.Background(), 0)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		err := r.UnlinkTreeClusterID(ctx, 1)

		// then
		assert.Error(t, err)
	})
}

func TestTreeRepository_UnlinkSensorID(t *testing.T) {
	t.Run("should unlink sensor ID successfully", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		// when
		err := r.UnlinkSensorID(context.Background(), "sensor-1")

		// then
		assert.NoError(t, err)

		tree, err := r.GetByID(context.Background(), 1)
		assert.NoError(t, err)
		assert.Nil(t, tree.Sensor, "Expected sensorID to be unlinked, but it is still linked")

	})
	t.Run("should return no error if sensor ID does not exist", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		err := r.UnlinkSensorID(context.Background(), "9999")

		// then
		assert.NoError(t, err)
	})
	t.Run("should return error when the context is canceled", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		invalidCtx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		err := r.UnlinkSensorID(invalidCtx, "sensor-1")

		// then
		assert.Error(t, err)
	})

	t.Run("should return error for empty sensor ID", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)

		// when
		err := r.UnlinkSensorID(context.Background(), "")

		// then
		assert.Error(t, err)
	})
}
