package region

import (
	"context"
	"errors"
	"log/slog"
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/storage/_mock"
	"github.com/stretchr/testify/assert"
)

var rootCtx = context.WithValue(context.Background(), "logger", slog.Default())

func TestRegionService_GetAll(t *testing.T) {
	t.Run("should return all regions", func(t *testing.T) {
		// given
		repo := storageMock.NewMockRegionRepository(t)
		svc := NewRegionService(repo)

		expectedRegions := []*entities.Region{
			{ID: 1, Name: "Region A"},
			{ID: 2, Name: "Region B"},
		}

		// when
		repo.EXPECT().GetAll(rootCtx).Return(expectedRegions, int64(len(expectedRegions)), nil)
		regions, totalCount, err := svc.GetAll(rootCtx)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedRegions, regions)
		assert.Equal(t, int64(len(expectedRegions)), totalCount)
	})

	t.Run("should return error when repository fails", func(t *testing.T) {
		// given
		repo := storageMock.NewMockRegionRepository(t)
		svc := NewRegionService(repo)
		expectedErr := errors.New("GetAll failed")

		repo.EXPECT().GetAll(rootCtx).Return(nil, int64(0), expectedErr)
		regions, totalCount, err := svc.GetAll(rootCtx)

		// then
		assert.Error(t, err)
		assert.Nil(t, regions)
		assert.Equal(t, int64(0), totalCount)
		//assert.EqualError(t, err, "500: GetAll failed")
	})
}

func TestRegionService_GetByID(t *testing.T) {
	t.Run("should return region when found", func(t *testing.T) {
		// given
		repo := storageMock.NewMockRegionRepository(t)
		svc := NewRegionService(repo)

		expectedRegion := &entities.Region{ID: 1, Name: "Region A"}

		// when
		repo.EXPECT().GetByID(rootCtx, int32(1)).Return(expectedRegion, nil)
		region, err := svc.GetByID(rootCtx, 1)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedRegion, region)
	})

	t.Run("should return error when region not found", func(t *testing.T) {
		// given
		repo := storageMock.NewMockRegionRepository(t)
		svc := NewRegionService(repo)

		// when
		repo.EXPECT().GetByID(rootCtx, int32(3)).Return(nil, storage.ErrEntityNotFound(""))
		region, err := svc.GetByID(rootCtx, 3)

		// then
		assert.Nil(t, region)
		assert.Error(t, err)
		//assert.EqualError(t, err, "404: region not found")
	})
}

func TestReady(t *testing.T) {
	t.Run("should return true if the service is ready", func(t *testing.T) {
		// given
		repo := storageMock.NewMockRegionRepository(t)
		svc := NewRegionService(repo)

		// when
		ready := svc.Ready()

		// then
		assert.True(t, ready)
	})

	t.Run("should return false if the service is not ready", func(t *testing.T) {
		// given
		svc := NewRegionService(nil)

		// when
		ready := svc.Ready()

		// then
		assert.False(t, ready)
	})
}
