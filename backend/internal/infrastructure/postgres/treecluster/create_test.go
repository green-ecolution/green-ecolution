package treecluster

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func TestTreeClusterRepository_Create(t *testing.T) {
	t.Run("should create tree cluster with name", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, "test", got.Name)
		assert.NotZero(t, got.ID)
		assert.WithinDuration(t, got.CreatedAt, time.Now(), time.Second)
		assert.WithinDuration(t, got.UpdatedAt, time.Now(), time.Second)
		assert.Nil(t, got.RegionID)
		assert.Empty(t, got.TreeIDs)
		assert.Equal(t, "", got.Address)
		assert.Equal(t, "", got.Description)
		assert.Equal(t, 0.0, got.MoistureLevel)
		assert.Nil(t, got.Coordinate)
		assert.Equal(t, shared.WateringStatusUnknown, got.WateringStatus)
		assert.Equal(t, cluster.TreeSoilConditionUnknown, got.SoilCondition)
		assert.False(t, got.Archived)
		assert.Nil(t, got.LastWatered)
	})

	t.Run("should create tree cluster with all values set", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		regionID := int32(1)
		coord := shared.MustNewCoordinate(54.3, 9.5)
		totalCountTree, _ := suite.Store.GetAllTreesCount(context.Background(), &sqlc.GetAllTreesCountParams{})
		testTrees, err := suite.Store.GetAllTrees(context.Background(), &sqlc.GetAllTreesParams{
			Limit:  int32(totalCountTree),
			Offset: 0,
		})
		assert.NoError(t, err)
		trees, err := mappers.treeMapper.FromSqlList(testTrees) // [0:2]
		if err != nil {
			t.Fatal(err)
		}

		trees = trees[0:2]
		treeIDs := utils.Map(trees, func(t *tree.Tree) int32 { return t.ID })

		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			tc.Address = "address"
			tc.Description = "description"
			tc.MoistureLevel = 1.0
			tc.WateringStatus = shared.WateringStatusGood
			tc.SoilCondition = cluster.TreeSoilConditionSchluffig
			tc.RegionID = &regionID
			tc.Coordinate = &coord
			tc.TreeIDs = treeIDs
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, "test", got.Name)
		assert.NotZero(t, got.ID)
		assert.WithinDuration(t, got.CreatedAt, time.Now(), time.Second)
		assert.WithinDuration(t, got.UpdatedAt, time.Now(), time.Second)
		assert.NotNil(t, got.RegionID)
		assert.Equal(t, regionID, *got.RegionID)
		assert.Equal(t, "address", got.Address)
		assert.Equal(t, "description", got.Description)
		assert.Equal(t, 1.0, got.MoistureLevel)
		assert.NotNil(t, got.Coordinate)
		assert.Equal(t, coord.Latitude(), got.Coordinate.Latitude())
		assert.Equal(t, coord.Longitude(), got.Coordinate.Longitude())
		assert.Equal(t, shared.WateringStatusGood, got.WateringStatus)
		assert.Equal(t, cluster.TreeSoilConditionSchluffig, got.SoilCondition)
		assert.False(t, got.Archived)
		assert.Nil(t, got.LastWatered)
		assert.NotNil(t, got.TreeIDs)
		assert.Len(t, got.TreeIDs, len(treeIDs))
		assert.Equal(t, treeIDs[0], got.TreeIDs[0])
		assert.Equal(t, treeIDs[1], got.TreeIDs[1])
	})

	t.Run("should return tree cluster with trees and link tree cluster id to trees", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		totalCountTree, _ := suite.Store.GetAllTreesCount(context.Background(), &sqlc.GetAllTreesCountParams{})
		testTrees, err := suite.Store.GetAllTrees(context.Background(), &sqlc.GetAllTreesParams{
			Provider: "",
			Limit:    int32(totalCountTree),
			Offset:   0,
		})

		if err != nil {
			t.Fatal(err)
		}
		trees, err := mappers.treeMapper.FromSqlList(testTrees) // [0:2]
		if err != nil {
			t.Fatal(err)
		}

		trees = trees[0:2]
		treeIDs := utils.Map(trees, func(t *tree.Tree) int32 { return t.ID })
		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			tc.TreeIDs = treeIDs
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)
		assert.NoError(t, err)

		sqlGotTrees, err := suite.Store.GetTreesByTreeClusterID(context.Background(), utils.P(got.ID))
		assert.NoError(t, err)

		assert.Len(t, sqlGotTrees, 2)
		assert.Equal(t, "test", got.Name)
		assert.NotZero(t, got.ID)
		assert.NotEmpty(t, got.TreeIDs)

		for i, tree := range sqlGotTrees {
			assert.Equal(t, treeIDs[i], tree.ID)
			assert.NotNil(t, tree.TreeClusterID)
			assert.Equal(t, got.ID, *tree.TreeClusterID)
		}
	})

	t.Run("should return tree cluster with coordinate", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		coord := shared.MustNewCoordinate(54.81269326939148, 9.484419532963013)
		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			tc.Coordinate = &coord
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, "test", got.Name)
		assert.NotZero(t, got.ID)
		assert.NotNil(t, got.Coordinate)
		assert.Equal(t, 54.81269326939148, got.Coordinate.Latitude())
		assert.Equal(t, 9.484419532963013, got.Coordinate.Longitude())
	})

	t.Run("should return error when tree cluster is invalid", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		// when
		got, err := r.Create(context.Background(), nil)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when tree cluster has empty name", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = ""
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error if context is canceled", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			return true, nil
		}

		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.Create(ctx, createFn)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when createFn returns error", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			return false, assert.AnError
		}

		tc, err := r.Create(context.Background(), createFn)
		assert.Error(t, err)
		assert.Nil(t, tc)
	})

	t.Run("should not create tree cluster when createFn returns false", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			return false, nil
		}

		// when
		tc, err := r.Create(context.Background(), createFn)

		// then
		assert.NoError(t, err)
		assert.Nil(t, tc)
	})

	t.Run("should rollback transaction when createFn returns false and not return error", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		newID := int32(9)

		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			return false, nil
		}

		// when
		tc, err := r.Create(context.Background(), createFn)
		got, _ := suite.Store.GetTreeClusterByID(context.Background(), newID)

		// then
		assert.NoError(t, err)
		assert.Nil(t, tc)
		assert.Empty(t, got)
	})

	t.Run("should rollback transaction when createFn returns error and return error", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		newID := int32(9)

		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			return false, assert.AnError
		}

		// when
		tc, err := r.Create(context.Background(), createFn)
		got, _ := suite.Store.GetTreeClusterByID(context.Background(), newID)

		// then
		assert.Error(t, err)
		assert.Nil(t, tc)
		assert.Empty(t, got)
	})
}

func TestTreeClusterRepository_LinkTreesToCluster(t *testing.T) {
	t.Run("should link trees to tree cluster", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			return true, nil
		}

		totalCountTree, _ := suite.Store.GetAllTreesCount(context.Background(), &sqlc.GetAllTreesCountParams{})
		testTrees, err := suite.Store.GetAllTrees(context.Background(), &sqlc.GetAllTreesParams{
			Provider: "",
			Limit:    int32(totalCountTree),
			Offset:   0,
		})
		assert.NoError(t, err)
		trees, err := mappers.treeMapper.FromSqlList(testTrees) // [0:2]
		if err != nil {
			t.Fatal(err)
		}

		trees = trees[0:2]

		tc, err := r.Create(context.Background(), createFn)
		assert.NoError(t, err)

		// when
		err = r.LinkTreesToCluster(context.Background(), tc.ID, utils.Map(trees, func(t *tree.Tree) int32 {
			return t.ID
		}))
		assert.NoError(t, err)

		// then
		for _, tree := range testTrees[0:2] {
			// before
			if tree.TreeClusterID != nil {
				assert.NotEqual(t, tc.ID, *tree.TreeClusterID)
			}

			// after
			sqlTree, err := suite.Store.GetTreeByID(context.Background(), tree.ID)
			assert.NoError(t, err)
			assert.NotNil(t, sqlTree.TreeClusterID)
			assert.Equal(t, tc.ID, *sqlTree.TreeClusterID)
		}
	})

	t.Run("should return error when tree cluster not found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		totalCountTree, _ := suite.Store.GetAllTreesCount(context.Background(), &sqlc.GetAllTreesCountParams{})
		testTrees, err := suite.Store.GetAllTrees(context.Background(), &sqlc.GetAllTreesParams{
			Provider: "",
			Limit:    int32(totalCountTree),
			Offset:   0,
		})
		assert.NoError(t, err)
		trees, err := mappers.treeMapper.FromSqlList(testTrees) // [0:2]
		if err != nil {
			t.Fatal(err)
		}

		trees = trees[0:2]

		// when
		err = r.LinkTreesToCluster(context.Background(), 99, utils.Map(trees, func(t *tree.Tree) int32 {
			return t.ID
		}))

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		totalCountTree, _ := suite.Store.GetAllTreesCount(context.Background(), &sqlc.GetAllTreesCountParams{})
		testTrees, err := suite.Store.GetAllTrees(context.Background(), &sqlc.GetAllTreesParams{
			Provider: "",
			Limit:    int32(totalCountTree),
			Offset:   0,
		})
		assert.NoError(t, err)
		trees, err := mappers.treeMapper.FromSqlList(testTrees) // [0:2]
		if err != nil {
			t.Fatal(err)
		}

		trees = trees[0:2]
		createFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			return true, nil
		}

		tc, err := r.Create(context.Background(), createFn)
		assert.NoError(t, err)

		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		err = r.LinkTreesToCluster(ctx, tc.ID, utils.Map(trees, func(t *tree.Tree) int32 {
			return t.ID
		}))

		// then
		assert.Error(t, err)
	})
}
