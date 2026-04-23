package tree

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

func TestTreeRepository_Create(t *testing.T) {
	t.Run("should create a tree with default values", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		// when
		got, err := r.Create(context.Background(), func(tree *shared.Tree, _ shared.TreeRepository) (bool, error) {
			return true, nil
		})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, "", got.Species)
		assert.NotZero(t, got.ID)
		assert.WithinDuration(t, got.CreatedAt, time.Now(), time.Second)
		assert.WithinDuration(t, got.UpdatedAt, time.Now(), time.Second)
		assert.Nil(t, got.TreeCluster)
		assert.Nil(t, got.Sensor)
		assert.Equal(t, "", got.Number)
		assert.Equal(t, int32(0), got.PlantingYear.Year())
		assert.Equal(t, float64(0), got.Coordinate.Latitude())
		assert.Equal(t, float64(0), got.Coordinate.Longitude())
		assert.Equal(t, "", got.Description)
		assert.Equal(t, "", got.Provider)
		assert.Equal(t, shared.WateringStatusUnknown, got.WateringStatus)
		assert.Nil(t, got.LastWatered)
	})

	t.Run("should create a tree with all values set", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := TreeRepository{store: suite.Store, TreeMappers: mappers}
		sqlTreeCluster, clusterErr := suite.Store.GetTreeClusterByID(context.Background(), 1)
		if clusterErr != nil {
			t.Fatal(clusterErr)
		}

		treeCluster, err := mappers.tcMapper.FromSql(sqlTreeCluster)
		if err != nil {
			t.Fatal(err)
		}

		sensorID := "sensor-1"
		sqlSensor, sensorErr := suite.Store.GetSensorByID(context.Background(), sensorID)
		if sensorErr != nil {
			t.Fatal(sensorErr)
		}

		sensor, err := mappers.sMapper.FromSql(sqlSensor)
		if err != nil {
			t.Fatal(err)
		}

		// when
		got, err := r.Create(context.Background(), func(tree *shared.Tree, _ shared.TreeRepository) (bool, error) {
			tree.Species = "Oak"
			tree.Number = "T001"
			tree.PlantingYear = shared.MustNewPlantingYear(2023)
			tree.Coordinate = shared.MustNewCoordinate(54.801539, 9.446741)
			tree.Description = "A newly planted oak tree"
			tree.WateringStatus = shared.WateringStatusGood
			tree.TreeCluster = treeCluster
			tree.Sensor = sensor
			return true, nil
		})

		treeClusterByTree, errClusterByTree := r.getTreeClusterByTreeID(context.Background(), got.ID)
		sensorByTree, errSensorByTree := r.GetSensorByTreeID(context.Background(), got.ID)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotZero(t, got.ID)
		assert.WithinDuration(t, got.CreatedAt, time.Now(), time.Second)
		assert.WithinDuration(t, got.UpdatedAt, time.Now(), time.Second)
		assert.NoError(t, errClusterByTree)
		assert.NotNil(t, treeClusterByTree)
		assert.Equal(t, treeCluster.ID, treeClusterByTree.ID)
		assert.NoError(t, errSensorByTree)
		assert.NotNil(t, sensorByTree)
		assert.Equal(t, sensor.ID, sensorByTree.ID)
		assert.Equal(t, "Oak", got.Species)
		assert.Equal(t, "T001", got.Number)
		assert.Equal(t, int32(2023), got.PlantingYear.Year())
		assert.Equal(t, 54.801539, got.Coordinate.Latitude())
		assert.Equal(t, 9.446741, got.Coordinate.Longitude())
		assert.Equal(t, "A newly planted oak tree", got.Description)
		assert.Equal(t, "", got.Provider)
		assert.Equal(t, shared.WateringStatusGood, got.WateringStatus)
		assert.Nil(t, got.LastWatered)
	})

	t.Run("should return error if latitude is out of bounds", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		// when
		got, err := r.Create(context.Background(), func(tree *shared.Tree, _ shared.TreeRepository) (bool, error) {
			coord, coordErr := shared.NewCoordinate(-200, 0)
			if coordErr != nil {
				return false, coordErr
			}
			tree.Coordinate = coord
			return true, nil
		})

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), shared.ErrInvalidLatitude.Error())
		assert.Nil(t, got)
	})

	t.Run("should return error if longitude is out of bounds", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		// when
		got, err := r.Create(context.Background(), func(tree *shared.Tree, _ shared.TreeRepository) (bool, error) {
			coord, coordErr := shared.NewCoordinate(0, 200)
			if coordErr != nil {
				return false, coordErr
			}
			tree.Coordinate = coord
			return true, nil
		})

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), shared.ErrInvalidLongitude.Error())
		assert.Nil(t, got)
	})

	t.Run("should return error if context is canceled", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)

		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.Create(ctx, func(tree *shared.Tree, _ shared.TreeRepository) (bool, error) {
			tree.Species = "Oak"
			return true, nil
		})

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}
