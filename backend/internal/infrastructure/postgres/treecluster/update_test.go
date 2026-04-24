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

		entity := &cluster.TreeCluster{
			Name:           "updated",
			Address:        "updated",
			Description:    "updated",
			MoistureLevel:  4.2,
			WateringStatus: shared.WateringStatusBad,
			SoilCondition:  cluster.TreeSoilConditionLehmig,
			Archived:       true,
			RegionID:       &regionID,
			LastWatered:    &now,
			Coordinate:     &coord,
			TreeIDs:        treeIDs,
		}

		// when
		updateErr := r.Update(context.Background(), 1, entity)
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
		assert.NotEmpty(t, got.TreeIDs)
		for _, tree := range testTrees[0:2] {
			assert.Equal(t, *tree.TreeClusterID, got.ID)
		}
	})

	t.Run("should return error when update tree cluster with non-existing id", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		entity := &cluster.TreeCluster{
			Name:           "updated",
			WateringStatus: shared.WateringStatusUnknown,
			SoilCondition:  cluster.TreeSoilConditionUnknown,
		}

		// when
		err := r.Update(context.Background(), 99, entity)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error when update tree cluster with negative id", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		entity := &cluster.TreeCluster{
			Name:           "updated",
			WateringStatus: shared.WateringStatusUnknown,
			SoilCondition:  cluster.TreeSoilConditionUnknown,
		}

		// when
		err := r.Update(context.Background(), -1, entity)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error if context is canceled", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()
		entity := &cluster.TreeCluster{
			Name:           "updated",
			WateringStatus: shared.WateringStatusUnknown,
			SoilCondition:  cluster.TreeSoilConditionUnknown,
		}

		// when
		err := r.Update(ctx, 1, entity)

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

		// Pass back the existing entity unchanged
		entity := &cluster.TreeCluster{
			Name:           gotBefore.Name,
			Address:        gotBefore.Address,
			Description:    gotBefore.Description,
			MoistureLevel:  gotBefore.MoistureLevel,
			WateringStatus: gotBefore.WateringStatus,
			SoilCondition:  gotBefore.SoilCondition,
			Archived:       gotBefore.Archived,
			RegionID:       gotBefore.RegionID,
			LastWatered:    gotBefore.LastWatered,
			Coordinate:     gotBefore.Coordinate,
			TreeIDs:        gotBefore.TreeIDs,
		}

		// when
		updateErr := r.Update(context.Background(), 1, entity)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.Equal(t, gotBefore.Name, got.Name)
		assert.Equal(t, gotBefore.Address, got.Address)
		assert.Equal(t, gotBefore.Description, got.Description)
		assert.Equal(t, gotBefore.MoistureLevel, got.MoistureLevel)
		assert.Equal(t, gotBefore.WateringStatus, got.WateringStatus)
		assert.Equal(t, gotBefore.SoilCondition, got.SoilCondition)
		assert.Equal(t, gotBefore.Archived, got.Archived)
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

		existing, err := r.GetByID(context.Background(), 1)
		assert.NoError(t, err)

		entity := &cluster.TreeCluster{
			Name:           existing.Name,
			Address:        existing.Address,
			Description:    existing.Description,
			MoistureLevel:  existing.MoistureLevel,
			WateringStatus: existing.WateringStatus,
			SoilCondition:  existing.SoilCondition,
			Archived:       existing.Archived,
			RegionID:       existing.RegionID,
			LastWatered:    existing.LastWatered,
			Coordinate:     existing.Coordinate,
			TreeIDs:        treeIDs,
		}

		// when
		updateErr := r.Update(context.Background(), 1, entity)
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

		entity := &cluster.TreeCluster{
			Name:           beforeTreeCluster.Name,
			Address:        beforeTreeCluster.Address,
			Description:    beforeTreeCluster.Description,
			MoistureLevel:  beforeTreeCluster.MoistureLevel,
			WateringStatus: beforeTreeCluster.WateringStatus,
			SoilCondition:  beforeTreeCluster.SoilCondition,
			Archived:       beforeTreeCluster.Archived,
			RegionID:       beforeTreeCluster.RegionID,
			LastWatered:    beforeTreeCluster.LastWatered,
			Coordinate:     beforeTreeCluster.Coordinate,
			TreeIDs:        []int32{}, // empty = unlink all
		}

		// when
		updateErr := r.Update(context.Background(), 1, entity)
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
		coord := shared.MustNewCoordinate(1.0, 1.0)

		existing, err := r.GetByID(context.Background(), 1)
		assert.NoError(t, err)

		entity := &cluster.TreeCluster{
			Name:           existing.Name,
			Address:        existing.Address,
			Description:    existing.Description,
			MoistureLevel:  existing.MoistureLevel,
			WateringStatus: existing.WateringStatus,
			SoilCondition:  existing.SoilCondition,
			Archived:       existing.Archived,
			RegionID:       existing.RegionID,
			LastWatered:    existing.LastWatered,
			TreeIDs:        existing.TreeIDs,
			Coordinate:     &coord,
		}

		// when
		updateErr := r.Update(context.Background(), 1, entity)
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

		existing, err := r.GetByID(context.Background(), 1)
		assert.NoError(t, err)

		entity := &cluster.TreeCluster{
			Name:           existing.Name,
			Address:        existing.Address,
			Description:    existing.Description,
			MoistureLevel:  existing.MoistureLevel,
			WateringStatus: existing.WateringStatus,
			SoilCondition:  existing.SoilCondition,
			Archived:       existing.Archived,
			RegionID:       existing.RegionID,
			LastWatered:    existing.LastWatered,
			TreeIDs:        existing.TreeIDs,
			Coordinate:     nil, // remove coordinates
		}

		// when
		updateErr := r.Update(context.Background(), 1, entity)
		got, getErr := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, updateErr)
		assert.NoError(t, getErr)
		assert.NotNil(t, got)
		assert.Nil(t, got.Coordinate)
	})

	t.Run("should return error when entity is nil", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)

		// when
		err := r.Update(context.Background(), 1, nil)

		// then
		assert.Error(t, err)
	})
}
