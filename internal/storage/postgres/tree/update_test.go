package tree

import (
	"context"
	"testing"
	"time"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/storage"
	"github.com/stretchr/testify/assert"
)

func TestTreeRepository_Update(t *testing.T) {
	t.Run("should update tree successfully", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/tree")
		r := NewTreeRepository(suite.Store, mappers)
		treeID := int32(1)
		date := time.Date(2024, 11, 22, 0, 0, 0, 0, time.UTC)

		newSpecies := "Updated Species"
		newNumber := "UpdatedNumber"
		newLatitude := 55.123456
		newLongitude := 10.654321
		newPlantingYear := int32(2025)
		newDescription := "Updated description"
		newWateringStatus := entities.WateringStatusGood
		newLastWateredValue := &date
		newProvider := "foo-provider"

		// when
		updatedTree, err := r.Update(context.Background(), treeID, func(tree *entities.Tree, _ storage.TreeRepository) (bool, error) {
			tree.Species = newSpecies
			tree.Number = newNumber
			tree.Latitude = newLatitude
			tree.Longitude = newLongitude
			tree.PlantingYear = newPlantingYear
			tree.Provider = newProvider
			tree.Description = newDescription
			tree.WateringStatus = newWateringStatus
			tree.LastWatered = newLastWateredValue
			return true, nil
		})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, updatedTree)
		assert.Equal(t, newSpecies, updatedTree.Species, "Species should match")
		assert.Equal(t, newNumber, updatedTree.Number, "Tree Number should match")
		assert.Equal(t, newLatitude, updatedTree.Latitude, "Latitude should match")
		assert.Equal(t, newLongitude, updatedTree.Longitude, "Longitude should match")
		assert.Equal(t, newPlantingYear, updatedTree.PlantingYear, "Planting Year should match")
		assert.Equal(t, newProvider, updatedTree.Provider, "Provider should match")
		assert.Equal(t, newDescription, updatedTree.Description, "Description should match")
		assert.Equal(t, newWateringStatus, updatedTree.WateringStatus, "Watering Status should match")
		assert.Equal(t, newLastWateredValue, updatedTree.LastWatered, "Last watered should match")
	})

	t.Run("should return error when tree is not found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		updatedTree, err := r.Update(context.Background(), int32(99), func(tree *entities.Tree, _ storage.TreeRepository) (bool, error) {
			tree.Species = "Non-existent species"
			return true, nil
		})

		// then
		assert.Error(t, err)
		assert.Nil(t, updatedTree)
	})

	t.Run("should return error the id is negative", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		updatedTree, err := r.Update(context.Background(), int32(-1), func(tree *entities.Tree, _ storage.TreeRepository) (bool, error) {
			tree.Species = "species"
			return true, nil
		})

		// then
		assert.Error(t, err)
		assert.Nil(t, updatedTree)
	})

	t.Run("should return error the id is zero", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeRepository(suite.Store, mappers)

		// when
		updatedTree, err := r.Update(context.Background(), int32(0), func(tree *entities.Tree, _ storage.TreeRepository) (bool, error) {
			tree.Species = "species"
			return true, nil
		})

		// then
		assert.Error(t, err)
		assert.Nil(t, updatedTree)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		updatedTree, err := r.Update(ctx, int32(1), func(tree *entities.Tree, _ storage.TreeRepository) (bool, error) {
			tree.Species = "Canceled context species"
			return true, nil
		})

		// then
		assert.Error(t, err)
		assert.Nil(t, updatedTree)
	})
}
