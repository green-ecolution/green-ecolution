package tree

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"

	sensorDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	treeDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func TestTreeRepository_GetAll(t *testing.T) {
	t.Run("should return all trees successfully without limitation", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		trees, totalCount, err := r.GetAll(ctx, treeDomain.TreeQuery{})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, trees)
		assert.NotEmpty(t, trees)
		assert.Equal(t, totalCount, int64(len(testTrees)))
		for i, tr := range trees {
			assertExpectedEqualToTree(t, testTrees[i], tr)
		}
	})

	t.Run("should return all trees successfully with provider", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		expectedTree := testTrees[len(testTrees)-1]

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAll(ctx, treeDomain.TreeQuery{Query: shared.Query{Provider: "test-provider"}})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Equal(t, totalCount, int64(1))
		assert.Equal(t, expectedTree.ID, got[0].ID, "ID does not match")
		assert.Equal(t, expectedTree.PlantingYear.Year(), got[0].PlantingYear.Year(), "PlantingYear does not match")
		assert.Equal(t, expectedTree.Species, got[0].Species, "Species does not match")
		assert.Equal(t, expectedTree.Number, got[0].Number, "Number does not match")
		assert.Equal(t, expectedTree.Coordinate.Latitude(), got[0].Coordinate.Latitude(), "Latitude does not match")
		assert.Equal(t, expectedTree.Coordinate.Longitude(), got[0].Coordinate.Longitude(), "Longitude does not match")
		assert.Equal(t, expectedTree.WateringStatus, got[0].WateringStatus, "WateringStatus does not match")
		assert.Equal(t, expectedTree.Description, got[0].Description, "Description does not match")
		assert.Equal(t, expectedTree.Provider, got[0].Provider, "Provider does not match")
		assert.Equal(t, expectedTree.AdditionalInfo, got[0].AdditionalInfo, "AdditionalInfo does not match")
	})

	t.Run("should return all trees successfully limited by 2 and with an offset of 2", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		exptectedTrees := testTrees[2:4]

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		trees, totalCount, err := r.GetAll(ctx, treeDomain.TreeQuery{})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, trees)
		assert.NotEmpty(t, trees)
		assert.Equal(t, totalCount, int64(len(testTrees)))
		assert.Len(t, trees, 2)
		for i, got := range trees {
			assert.Equal(t, exptectedTrees[i].ID, got.ID, "ID does not match")
		}
	})

	t.Run("should return error on invalid page value", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(0))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, treeDomain.TreeQuery{})

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error on invalid limit value", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(0))

		// when
		got, totalCount, err := r.GetAll(ctx, treeDomain.TreeQuery{})

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return empty list when no trees exist", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAll(ctx, treeDomain.TreeQuery{})

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error if context is canceled", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		trees, totalCount, err := r.GetAll(ctx, treeDomain.TreeQuery{})

		// then
		assert.Error(t, err)
		assert.Nil(t, trees)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return trees filtered by watering status and planting years", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		filteredTrees := []*treeDomain.Tree{testTrees[1], testTrees[2]}

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		trees, totalCount, err := r.GetAll(ctx, treeDomain.TreeQuery{
			WateringStatuses: []shared.WateringStatus{shared.WateringStatusBad, shared.WateringStatusGood},
			PlantingYears:    []int32{2022, 2023},
			HasCluster:       utils.P(true),
		})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, trees)
		assert.NotEmpty(t, trees)
		assert.Equal(t, totalCount, int64(len(filteredTrees)))
		for i, tr := range trees {
			assertExpectedEqualToTree(t, filteredTrees[i], tr)
		}
	})
}

func TestTreeRepository_GetCount(t *testing.T) {
	t.Run("should return count of all trees in db", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		// when
		totalCount, err := r.GetCount(context.Background(), treeDomain.TreeQuery{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, int64(len(testTrees)), totalCount)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		totalCount, err := r.GetCount(ctx, treeDomain.TreeQuery{})

		// then
		assert.Error(t, err)
		assert.Equal(t, int64(0), totalCount)
	})
}

func TestTreeRepository_GetByID(t *testing.T) {
	t.Run("should return the correct tree by ID", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		treeID := int32(1)

		sqlTreeCluster, clusterErr := suite.Store.GetTreeClusterByTreeID(context.Background(), treeID)
		if clusterErr != nil {
			t.Fatal(clusterErr)
		}
		treeCluster, err := mappers.tcMapper.FromSql(sqlTreeCluster)
		if err != nil {
			t.Fatal(err)
		}

		sqlSensor, sensorErr := suite.Store.GetSensorByTreeID(context.Background(), treeID)
		if sensorErr != nil {
			t.Fatal(sensorErr)
		}
		sn, err := mappers.sMapper.FromSql(sqlSensor)
		if err != nil {
			t.Fatal(err)
		}

		// when
		tr, err := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, tr)
		assert.NotNil(t, tr.TreeClusterID)
		assert.Equal(t, treeCluster.ID, *tr.TreeClusterID)
		assert.NotNil(t, tr.SensorID)
		assert.Equal(t, sn.ID, *tr.SensorID)
		assertExpectedEqualToTree(t, tr, testTrees[0])
	})

	t.Run("should return error if tree not found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		tr, err := r.GetByID(context.Background(), 99)

		// then
		assert.Error(t, err)
		assert.Nil(t, tr)
	})

	t.Run("should return error if tree id is negative", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		tr, err := r.GetByID(context.Background(), -1)

		// then
		assert.Error(t, err)
		assert.Nil(t, tr)
	})

	t.Run("should return error if tree id is zero", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		tr, err := r.GetByID(context.Background(), 0)

		// then
		assert.Error(t, err)
		assert.Nil(t, tr)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		tr, err := r.GetByID(ctx, 1)

		// then
		assert.Error(t, err)
		assert.Nil(t, tr)
	})
}

func TestTreeRepository_GetBySensorID(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")

	t.Run("should return the correct tree by linked sensor ID", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		sensorID := sensorDomain.MustNewSensorID("sensor-1")

		// when
		tr, err := r.GetBySensorID(context.Background(), sensorID)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, tr)
		assert.NotNil(t, tr.SensorID)
		assert.Equal(t, tr.SensorID.String(), sensorID.String())
		assertExpectedEqualToTree(t, tr, testTrees[0])
	})

	t.Run("should return error when sensor is not found", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		sensorID := sensorDomain.MustNewSensorID("sensor-notFound")

		// when
		tr, err := r.GetBySensorID(context.Background(), sensorID)

		// then
		assert.Error(t, err)
		assert.Nil(t, tr)
		// assert.Equal(t, "sensor not found", err.Error())
	})

	t.Run("should return error when tree is not found", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		sensorID := sensorDomain.MustNewSensorID("sensor-4")

		// when
		tr, err := r.GetBySensorID(context.Background(), sensorID)

		// then
		assert.Error(t, err)
		assert.Nil(t, tr)
		// assert.Equal(t, "entity not found", err.Error())
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()
		sensorID := sensorDomain.MustNewSensorID("sensor-1")

		// when
		trees, err := r.GetBySensorID(ctx, sensorID)

		// then
		assert.Error(t, err)
		assert.Nil(t, trees)
	})
}

func TestTreeRepository_GetBySensorIDs(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")

	t.Run("should return sensor by multiple ids", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)

		// when
		got, err := r.GetBySensorIDs(context.Background(), sensorDomain.MustNewSensorID("sensor-1"), sensorDomain.MustNewSensorID("sensor-2"))

		// then
		assert.NoError(t, err)
		assert.Len(t, got, 2)
	})

	t.Run("should return empty list when sensors is not found", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)

		// when
		got, err := r.GetBySensorIDs(context.Background(), sensorDomain.MustNewSensorID("sensor-notFound"), sensorDomain.MustNewSensorID("sensor-notExists"))

		// then
		assert.NoError(t, err)
		assert.Len(t, got, 0)
	})

	t.Run("should return found sensors one min one id exists", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)

		// when
		got, err := r.GetBySensorIDs(context.Background(), sensorDomain.MustNewSensorID("sensor-1"), sensorDomain.MustNewSensorID("sensor-notExists"))

		// then
		assert.NoError(t, err)
		assert.Len(t, got, 1)
		assert.NotNil(t, got[0].SensorID)
		assert.Equal(t, "sensor-1", got[0].SensorID.String())
	})
}

func TestTreeRepository_GetTreesByIDs(t *testing.T) {
	t.Run("should return trees successfully by IDs", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		ids := []int32{1, 2}
		expectedTrees := testTrees[:2]

		// when
		trees, err := r.GetTreesByIDs(context.Background(), ids)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, trees)
		assert.Len(t, trees, len(expectedTrees))
		for i, tr := range trees {
			assertExpectedEqualToTree(t, expectedTrees[i], tr)
		}
	})

	t.Run("should return empty list if no trees are found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)
		ids := []int32{99, 100, -1, 0}

		// when
		trees, err := r.GetTreesByIDs(context.Background(), ids)

		// then
		assert.NoError(t, err)
		assert.Empty(t, trees)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		trees, err := r.GetTreesByIDs(ctx, []int32{1, 2})

		// then
		assert.Error(t, err)
		assert.Nil(t, trees)
	})
}

func TestTreeRepository_GetByTreeClusterID(t *testing.T) {
	t.Run("should return trees successfully for a given tree cluster ID", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		treeClusterID := int32(1)
		expectedTrees := testTrees[:2]

		// when
		trees, err := r.GetByTreeClusterID(context.Background(), treeClusterID)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, trees)
		assert.NotEmpty(t, trees)
		assert.Len(t, trees, len(expectedTrees))
		for i, tr := range trees {
			assertExpectedEqualToTree(t, expectedTrees[i], tr)
		}
	})

	t.Run("should return empty slice when tree cluster ID is not found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		trees, err := r.GetByTreeClusterID(context.Background(), 99)

		// then
		assert.NoError(t, err)
		assert.Empty(t, trees)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		trees, err := r.GetByTreeClusterID(ctx, 1)

		// then
		assert.Error(t, err)
		assert.Nil(t, trees)
	})
}

func TestTreeRepository_GetByCoordinates(t *testing.T) {
	t.Run("should return tree successfully for given coordinates", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		testTree := testTrees[0]

		// when
		tr, err := r.GetByCoordinates(context.Background(), testTree.Coordinate)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, tr)
		assertExpectedEqualToTree(t, testTree, tr)
	})

	t.Run("should return error when no tree is found for given coordinates", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		tr, err := r.GetByCoordinates(context.Background(), shared.MustNewCoordinate(0.0, 0.0))

		// then
		assert.Error(t, err)
		assert.Nil(t, tr)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		tr, err := r.GetByCoordinates(ctx, shared.MustNewCoordinate(54.821248093376, 9.485710628517))

		// then
		assert.Error(t, err)
		assert.Nil(t, tr)
	})
}

func TestTreeRepository_GetSensorByTreeID(t *testing.T) {
	t.Run("should return sensor for the given tree ID", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		treeID := int32(1)

		// when
		sn, err := r.GetSensorByTreeID(context.Background(), treeID)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, sn, "Sensor should not be nil")
		assert.NotZero(t, sn.ID, "Sensor ID should not be zero")
	})

	t.Run("should return ErrSensorNotFound when no sensor is linked to the tree", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)
		treeID := int32(2)

		// when
		sn, err := r.GetSensorByTreeID(context.Background(), treeID)

		// then
		assert.Error(t, err)
		assert.Nil(t, sn, "Sensor should be nil when not found")
	})

	t.Run("should return error when tree ID does not exist", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)
		invalidTreeID := int32(999)

		// when
		sn, err := r.GetSensorByTreeID(context.Background(), invalidTreeID)

		// then
		assert.Error(t, err, "Expected an error when the tree ID does not exist")
		assert.Nil(t, sn, "Sensor should be nil when tree ID does not exist")
	})

	t.Run("should return error when tree ID is negative", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)
		invalidTreeID := int32(-1)

		// when
		sn, err := r.GetSensorByTreeID(context.Background(), invalidTreeID)

		// then
		assert.Error(t, err, "Expected an error when the tree ID is negative")
		assert.Nil(t, sn, "Sensor should be nil when tree ID is negative")
	})

	t.Run("should return error when tree ID is zero", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)
		invalidTreeID := int32(0)

		// when
		sn, err := r.GetSensorByTreeID(context.Background(), invalidTreeID)

		// then
		assert.Error(t, err, "Expected an error when the tree ID ID is zero")
		assert.Nil(t, sn, "Sensor should be nil when tree ID ID is zero")
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		sn, err := r.GetSensorByTreeID(ctx, 1)

		// then
		assert.Error(t, err)
		assert.Nil(t, sn)
	})
}

func TestTreeRepository_GetTreeClusterByTreeID(t *testing.T) {
	t.Run("should return tree cluster for the given tree ID", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := TreeRepository{store: suite.Store, TreeMappers: mappers}
		treeID := int32(1)

		// when
		treeCluster, err := r.getTreeClusterByTreeID(context.Background(), treeID)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, treeCluster, "TreeCluster should not be nil")
		assert.NotZero(t, treeCluster.ID, "TreeCluster ID should not be zero")
	})

	t.Run("should return ErrTreeClusterNotFound when no tree cluster is linked to the tree", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := TreeRepository{store: suite.Store, TreeMappers: mappers}
		treeID := int32(4)

		// when
		treeCluster, err := r.getTreeClusterByTreeID(context.Background(), treeID)

		// then
		assert.Error(t, err)
		assert.Nil(t, treeCluster, "TreeCluster should be nil when not found")
	})

	t.Run("should return error when tree ID does not exist", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := TreeRepository{store: suite.Store, TreeMappers: mappers}
		invalidTreeID := int32(999)

		// when
		treeCluster, err := r.getTreeClusterByTreeID(context.Background(), invalidTreeID)

		// then
		assert.Error(t, err, "Expected an error when the tree ID does not exist")
		assert.Nil(t, treeCluster, "TreeCluster should be nil when tree ID does not exist")
	})

	t.Run("should return error when tree ID is zero", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := TreeRepository{store: suite.Store, TreeMappers: mappers}
		invalidTreeID := int32(0)

		// when
		treeCluster, err := r.getTreeClusterByTreeID(context.Background(), invalidTreeID)

		// then
		assert.Error(t, err, "Expected an error when the tree ID is zero")
		assert.Nil(t, treeCluster, "TreeCluster should be nil when tree ID is zero")
	})

	t.Run("should return error when tree ID is negative", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := TreeRepository{store: suite.Store, TreeMappers: mappers}
		invalidTreeID := int32(-1)

		// when
		treeCluster, err := r.getTreeClusterByTreeID(context.Background(), invalidTreeID)

		// then
		assert.Error(t, err, "Expected an error when the tree ID is negative")
		assert.Nil(t, treeCluster, "TreeCluster should be nil when tree ID is negative")
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := TreeRepository{store: suite.Store, TreeMappers: mappers}
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		treeCluster, err := r.getTreeClusterByTreeID(ctx, 1)

		// then
		assert.Error(t, err)
		assert.Nil(t, treeCluster)
	})
}

func TestTreeRepository_FindNearestTree(t *testing.T) {
	t.Run("should return the nearest tree for given latitude and longitude", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		// when
		tr, err := r.GetByID(context.Background(), 2)
		assert.NoError(t, err)
		assert.NotNil(t, tr)

		sensorLatitude := 54.821517
		sensorLongitude := 9.487169

		// when
		nearestTree, errFind := r.FindNearestTree(context.Background(), shared.MustNewCoordinate(sensorLatitude, sensorLongitude))

		// then
		assert.NoError(t, errFind, "Expected no error while finding the nearest tree")
		assert.NotNil(t, nearestTree, "Expected to find a nearest tree")
		assertExpectedEqualToTree(t, tr, nearestTree)
	})

	t.Run("should return error when no tree found within the required distance", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		sensorLatitude := 54.821535
		sensorLongitude := 9.487200

		// when
		nearestTree, err := r.FindNearestTree(context.Background(), shared.MustNewCoordinate(sensorLatitude, sensorLongitude))

		// then
		assert.Error(t, err, "Expected error while finding the nearest tree")
		assert.Nil(t, nearestTree, "no tree should be found")
	})

	t.Run("should return error if context is canceled", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		tr, err := r.FindNearestTree(ctx, shared.MustNewCoordinate(54.82124518093376, 9.485702120628517))

		// then
		assert.Error(t, err, "Expected error when context is canceled")
		assert.Nil(t, tr, "Expected no tree to be returned when context is canceled")
	})
}

func TestTreeRepository_FindNearestTrees(t *testing.T) {
	// Tree 1 (54.82124, 9.48570) and tree 2 (54.82150, 9.48715) are ~95m apart
	// in the Solitüde Strand seed cluster. A query near them should return both.
	queryLat := 54.821517
	queryLng := 9.487169

	t.Run("should return nearby trees sorted ascending by distance", func(t *testing.T) {
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		got, err := r.FindNearestTrees(context.Background(), shared.MustNewCoordinate(queryLat, queryLng), 500, 10)

		assert.NoError(t, err)
		assert.NotEmpty(t, got)
		for i := 1; i < len(got); i++ {
			assert.GreaterOrEqual(t, got[i].Distance.Meters(), got[i-1].Distance.Meters(), "results must be sorted by distance ascending")
			assert.NotNil(t, got[i].Tree)
		}
	})

	t.Run("should populate sensor ID on trees that have one assigned", func(t *testing.T) {
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		got, err := r.FindNearestTrees(context.Background(), shared.MustNewCoordinate(queryLat, queryLng), 500, 10)
		assert.NoError(t, err)

		var hasTreeWithSensor, hasTreeWithoutSensor bool
		for _, tw := range got {
			if tw.Tree.SensorID != nil {
				hasTreeWithSensor = true
			} else {
				hasTreeWithoutSensor = true
			}
		}
		assert.True(t, hasTreeWithSensor, "expected at least one tree with an assigned sensor in the result")
		assert.True(t, hasTreeWithoutSensor, "expected at least one tree without a sensor in the result")
	})

	t.Run("should respect max results limit", func(t *testing.T) {
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		got, err := r.FindNearestTrees(context.Background(), shared.MustNewCoordinate(queryLat, queryLng), 10000, 1)

		assert.NoError(t, err)
		assert.Len(t, got, 1)
	})

	t.Run("should return empty slice when no trees within radius", func(t *testing.T) {
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		got, err := r.FindNearestTrees(context.Background(), shared.MustNewCoordinate(0.0, 0.0), 100, 10)

		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		got, err := r.FindNearestTrees(ctx, shared.MustNewCoordinate(queryLat, queryLng), 500, 10)

		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func assertExpectedEqualToTree(t *testing.T, expectedTree, tr *treeDomain.Tree) {
	assert.Equal(t, expectedTree.ID, tr.ID, "ID does not match")
	assert.Equal(t, expectedTree.PlantingYear.Year(), tr.PlantingYear.Year(), "PlantingYear does not match")
	assert.Equal(t, expectedTree.Species, tr.Species, "Species does not match")
	assert.Equal(t, expectedTree.Number, tr.Number, "Number does not match")
	assert.Equal(t, expectedTree.Coordinate.Latitude(), tr.Coordinate.Latitude(), "Latitude does not match")
	assert.Equal(t, expectedTree.Coordinate.Longitude(), tr.Coordinate.Longitude(), "Longitude does not match")
	assert.Equal(t, expectedTree.WateringStatus, tr.WateringStatus, "WateringStatus does not match")
	assert.Equal(t, expectedTree.Description, tr.Description, "Description does not match")
	assert.Equal(t, expectedTree.Provider, tr.Provider, "Provider does not match")
	assert.Equal(t, expectedTree.AdditionalInfo, tr.AdditionalInfo, "AdditionalInfo does not match")
	assert.Equal(t, expectedTree.LastWatered, tr.LastWatered, "Last watered does not match")
}

var (
	testTrees = []*treeDomain.Tree{
		{
			ID:             1,
			PlantingYear:   treeDomain.MustNewPlantingYear(2021),
			Species:        "Quercus robur",
			Number:         "1005",
			Coordinate:     shared.MustNewCoordinate(54.82124518093376, 9.485702120628517),
			WateringStatus: "unknown",
			Description:    "Sample description 1",
			LastWatered:    nil,
		},
		{
			ID:             2,
			PlantingYear:   treeDomain.MustNewPlantingYear(2022),
			Species:        "Quercus robur",
			Number:         "1006",
			Coordinate:     shared.MustNewCoordinate(54.8215076622281, 9.487153277881877),
			WateringStatus: "good",
			Description:    "Sample description 2",
			LastWatered:    nil,
		},
		{
			ID:             3,
			PlantingYear:   treeDomain.MustNewPlantingYear(2023),
			Species:        "Betula pendula",
			Number:         "1007",
			Coordinate:     shared.MustNewCoordinate(54.78780993841013, 9.444052105200551),
			WateringStatus: "bad",
			Description:    "Sample description 3",
			LastWatered:    nil,
		},
		{
			ID:             4,
			PlantingYear:   treeDomain.MustNewPlantingYear(2020),
			Species:        "Quercus robur",
			Number:         "1008",
			Coordinate:     shared.MustNewCoordinate(54.1000, 9.2000),
			WateringStatus: "bad",
			Description:    "Sample description 4",
			LastWatered:    nil,
		},
		{
			ID:             5,
			PlantingYear:   treeDomain.MustNewPlantingYear(2022),
			Species:        "Betula pendula",
			Number:         "1009",
			Coordinate:     shared.MustNewCoordinate(54.22, 9.11),
			WateringStatus: "bad",
			Description:    "Sample description 5",
			Provider:       "test-provider",
			AdditionalInfo: map[string]interface{}{
				"foo": "bar",
			},
			LastWatered: nil,
		},
	}
)
