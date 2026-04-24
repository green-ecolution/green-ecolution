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

func TestTreeClusterRepository_Update(t *testing.T) {
	t.Run("should update tree cluster", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		regionID := int32(1)
		now := time.Now().UTC()
		coord := shared.MustNewCoordinate(54.3, 9.5)

		totalCountTree, _ := suite.Store.GetAllTreesCount(context.Background(), &sqlc.GetAllTreesCountParams{})
		testTrees, err := suite.Store.GetAllTrees(context.Background(), &sqlc.GetAllTreesParams{
			Limit:    int32(totalCountTree),
			Offset:   0,
			Provider: "",
		})
		assert.NoError(t, err)
		trees, err := mappers.treeMapper.FromSqlList(testTrees) // [0:2]
		if err != nil {
			t.Fatal(err)
		}

		trees = trees[0:2]
		treeIDs := utils.Map(trees, func(t *tree.Tree) int32 { return t.ID })

		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "updated"
			tc.Address = "updated"
			tc.Description = "updated"
			tc.MoistureLevel = 4.2
			tc.WateringStatus = shared.WateringStatusBad
			tc.Archived = true
			tc.RegionID = &regionID
			tc.LastWatered = &now
			tc.Coordinate = &coord
			tc.SoilCondition = cluster.TreeSoilConditionLehmig
			tc.TreeIDs = treeIDs
			return true, nil
		}

		// when
		updateErr := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.Equal(t, "updated", got.Name)
		assert.Equal(t, "updated", got.Address)
		assert.Equal(t, "updated", got.Description)
		assert.Equal(t, 4.2, got.MoistureLevel)
		assert.Equal(t, shared.WateringStatusBad, got.WateringStatus)
		assert.Equal(t, true, got.Archived)
		assert.NotNil(t, got.RegionID)
		assert.Equal(t, regionID, *got.RegionID)
		assert.NotNil(t, got.LastWatered)
		assert.WithinDuration(t, now, got.LastWatered.UTC(), time.Second)
		assert.NotNil(t, got.Coordinate)
		assert.Equal(t, coord.Latitude(), got.Coordinate.Latitude())
		assert.Equal(t, coord.Longitude(), got.Coordinate.Longitude())
		assert.Equal(t, cluster.TreeSoilConditionLehmig, got.SoilCondition)
		assert.NotNil(t, got.TreeIDs)
		assert.Len(t, got.TreeIDs, len(treeIDs))
		for _, tree := range testTrees[0:2] {
			assert.Equal(t, *tree.TreeClusterID, got.ID)
		}
	})

	t.Run("should return error when update tree cluster with non-existing id", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "updated"
			return true, nil
		}

		// when
		err := r.Update(context.Background(), 99, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when update tree cluster with negative id", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "updated"
			return true, nil
		}

		// when
		err := r.Update(context.Background(), -1, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error if context is canceled", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "updated"
			return true, nil
		}
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		err := r.Update(ctx, 1, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should not update tree cluster when no changes", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		gotBefore, err := r.GetByID(context.Background(), 1)
		assert.NoError(t, err)
		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			return false, nil
		}

		// when
		updateErr := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.Equal(t, gotBefore, got)
	})

	t.Run("should link trees to tree cluster", func(t *testing.T) {
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
		treeIDs := utils.Map(trees, func(t *tree.Tree) int32 { return t.ID })
		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.TreeIDs = treeIDs
			return true, nil
		}

		// when
		updateErr := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		for _, tree := range testTrees[0:2] {
			assert.Equal(t, got.ID, *tree.TreeClusterID)
		}
	})

	t.Run("should unlink trees from tree cluster", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		beforeTreeCluster, err := r.GetByID(context.Background(), 1)
		assert.NoError(t, err)
		beforeTreeIDs := beforeTreeCluster.TreeIDs

		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.TreeIDs = nil
			return true, nil
		}

		// when
		updateErr := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		for _, treeID := range beforeTreeIDs {
			actualTree, err := suite.Store.GetTreeByID(context.Background(), treeID)
			assert.NoError(t, err)
			assert.Nil(t, actualTree.TreeClusterID)
		}
	})

	t.Run("should update tree cluster coordinates", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		newCoord := shared.MustNewCoordinate(1.0, 1.0)
		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Coordinate = &newCoord
			return true, nil
		}

		// when
		updateErr := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotNil(t, got.Coordinate)
		assert.Equal(t, 1.0, got.Coordinate.Latitude())
		assert.Equal(t, 1.0, got.Coordinate.Longitude())
	})

	t.Run("should remove tree cluster coordinates", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Coordinate = nil
			return true, nil
		}

		// when
		updateErr := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.Nil(t, got.Coordinate)
	})

	t.Run("should return error when updateFn is nil", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)

		// when
		err := r.Update(context.Background(), 1, nil)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when updateFn returns error", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			return true, assert.AnError
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)

		// then
		assert.Error(t, err)
	})

	t.Run("should not update when updateFn returns false", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			return false, nil
		}

		// when
		updateErr := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
	})

	t.Run("should not rollback when updateFn returns false", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		updateFn := func(tc *cluster.TreeCluster, _ cluster.TreeClusterRepository) (bool, error) {
			tc.Name = "updated"
			return false, nil
		}

		// when
		err := r.Update(context.Background(), 1, updateFn)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, err)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.NotEqual(t, "updated", got.Name)
	})
}
