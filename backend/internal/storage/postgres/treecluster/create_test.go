package treecluster

import (
	"context"
	"testing"
	"time"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/storage"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
)

func TestTreeClusterRepository_Create(t *testing.T) {
	t.Run("should create tree cluster with name", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
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
		assert.Nil(t, got.Region)
		assert.Empty(t, got.Trees)
		assert.Equal(t, "", got.Address)
		assert.Equal(t, "", got.Description)
		assert.Equal(t, 0.0, got.MoistureLevel)
		assert.Nil(t, got.Latitude)
		assert.Nil(t, got.Longitude)
		assert.Equal(t, entities.WateringStatusUnknown, got.WateringStatus)
		assert.Equal(t, entities.TreeSoilConditionUnknown, got.SoilCondition)
		assert.False(t, got.Archived)
		assert.Nil(t, got.LastWatered)
	})

	t.Run("should create tree cluster with all values set", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		newRegion := &entities.Region{
			ID:        1,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
			Name:      "Mürwik",
		}
		lat := 54.3
		long := 9.5
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

		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			tc.Address = "address"
			tc.Description = "description"
			tc.MoistureLevel = 1.0
			tc.WateringStatus = entities.WateringStatusGood
			tc.SoilCondition = entities.TreeSoilConditionSchluffig
			tc.Region = newRegion
			tc.Latitude = &lat
			tc.Longitude = &long
			tc.Trees = trees
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
		assert.Equal(t, newRegion.ID, got.Region.ID)
		assert.Equal(t, newRegion.Name, got.Region.Name)
		assert.Equal(t, "address", got.Address)
		assert.Equal(t, "description", got.Description)
		assert.Equal(t, 1.0, got.MoistureLevel)
		assert.NotNil(t, got.Latitude)
		assert.NotNil(t, got.Longitude)
		assert.Equal(t, lat, *got.Latitude)
		assert.Equal(t, long, *got.Longitude)
		assert.Equal(t, entities.WateringStatusGood, got.WateringStatus)
		assert.Equal(t, entities.TreeSoilConditionSchluffig, got.SoilCondition)
		assert.False(t, got.Archived)
		assert.Nil(t, got.LastWatered)
		assert.NotNil(t, got.Trees)
		assert.Len(t, got.Trees, len(trees))
		assert.Equal(t, trees[0].ID, got.Trees[0].ID)
		assert.Equal(t, trees[1].ID, got.Trees[1].ID)
	})

	t.Run("should return tree cluster with trees and link tree cluster id to trees", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
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
		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			tc.Trees = trees
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
		assert.NotEmpty(t, got.Trees)

		for i, tree := range sqlGotTrees {
			assert.Equal(t, trees[i].ID, tree.ID)
			assert.NotNil(t, tree.TreeClusterID)
			assert.Equal(t, got.ID, *tree.TreeClusterID)
		}
	})

	t.Run("should return tree cluster with latitude and longitude", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			tc.Latitude = utils.P(54.81269326939148)
			tc.Longitude = utils.P(9.484419532963013)
			return true, nil
		}

		// when
		got, err := r.Create(context.Background(), createFn)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, "test", got.Name)
		assert.NotZero(t, got.ID)
		assert.NotNil(t, got.Latitude)
		assert.NotNil(t, got.Longitude)
		assert.Equal(t, 54.81269326939148, *got.Latitude)
		assert.Equal(t, 9.484419532963013, *got.Longitude)
	})

	t.Run("should return error when tree cluster is invalid", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
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
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
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
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
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
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
			return false, assert.AnError
		}

		tc, err := r.Create(context.Background(), createFn)
		assert.Error(t, err)
		assert.Nil(t, tc)
	})

	t.Run("should not create tree cluster when createFn returns false", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
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
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		newID := int32(9)

		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
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
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		newID := int32(9)

		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
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
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
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
		err = r.LinkTreesToCluster(context.Background(), tc.ID, utils.Map(trees, func(t *entities.Tree) int32 {
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
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
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
		err = r.LinkTreesToCluster(context.Background(), 99, utils.Map(trees, func(t *entities.Tree) int32 {
			return t.ID
		}))

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
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
		createFn := func(tc *entities.TreeCluster, _ storage.TreeClusterRepository) (bool, error) {
			tc.Name = "test"
			return true, nil
		}

		tc, err := r.Create(context.Background(), createFn)
		assert.NoError(t, err)

		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		err = r.LinkTreesToCluster(ctx, tc.ID, utils.Map(trees, func(t *entities.Tree) int32 {
			return t.ID
		}))

		// then
		assert.Error(t, err)
	})
}
