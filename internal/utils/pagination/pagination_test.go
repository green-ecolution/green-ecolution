package pagination

import (
	"context"
	"testing"

	"github.com/green-ecolution/backend/internal/server/http/entities"
	"github.com/stretchr/testify/assert"
)

func TestPaginationUtil_GetValues(t *testing.T) {
	t.Run("should return page and limit", func(t *testing.T) {
		expectedPage := int32(1)
		expectedLimit := int32(-1)

		ctx := context.WithValue(context.Background(), "page", expectedPage)
		ctx = context.WithValue(ctx, "limit", expectedLimit)

		page, limit, err := GetValues(ctx)

		assert.Nil(t, err)
		assert.Equal(t, page, expectedPage)
		assert.Equal(t, limit, expectedLimit)
	})

	t.Run("should return error on invalid page", func(t *testing.T) {
		expectedPage := int32(-1)
		expectedLimit := int32(10)

		ctx := context.WithValue(context.Background(), "page", expectedPage)
		ctx = context.WithValue(ctx, "limit", expectedLimit)

		_, _, err := GetValues(ctx)

		assert.NotNil(t, err)
	})

	t.Run("should return error on invalid limit", func(t *testing.T) {
		expectedPage := int32(10)
		expectedLimit := int32(-10)

		ctx := context.WithValue(context.Background(), "page", expectedPage)
		ctx = context.WithValue(ctx, "limit", expectedLimit)

		_, _, err := GetValues(ctx)

		assert.NotNil(t, err)
	})

	t.Run("should return default values on empty context", func(t *testing.T) {
		expectedPage := int32(1)
		expectedLimit := int32(-1)

		page, limit, err := GetValues(context.Background())

		assert.Equal(t, expectedPage, page)
		assert.Equal(t, expectedLimit, limit)
		assert.NoError(t, err)
	})
}

func TestPaginationUtil_Create(t *testing.T) {
	t.Run("should return a valid pagination", func(t *testing.T) {
		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(10))

		totalCount := int64(50)
		pagination := Create(ctx, totalCount)

		expectedNextPage := int32(3)
		expectedPrevPage := int32(1)
		expectedPagination := &entities.Pagination{
			Total:       totalCount,
			CurrentPage: 2,
			TotalPages:  5,
			NextPage:    &expectedNextPage,
			PrevPage:    &expectedPrevPage,
		}

		// assert pagination values
		assert.NotNil(t, pagination)
		assert.Equal(t, expectedPagination.Total, pagination.Total)
		assert.Equal(t, expectedPagination.CurrentPage, pagination.CurrentPage)
		assert.Equal(t, expectedPagination.TotalPages, pagination.TotalPages)
		assert.Equal(t, expectedPagination.NextPage, pagination.NextPage)
		assert.Equal(t, expectedPagination.PrevPage, pagination.PrevPage)
	})

	t.Run("should return nil on limit with value -1", func(t *testing.T) {
		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		pagination := Create(ctx, int64(50))

		assert.Nil(t, pagination)
	})

	t.Run("should return nil on no context values", func(t *testing.T) {
		pagination := Create(context.Background(), int64(50))

		assert.Nil(t, pagination)
	})
}
