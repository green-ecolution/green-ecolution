package region_test

import (
	"context"
	"errors"
	"net/http"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/region"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/middleware"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/service/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestGetAllRegions(t *testing.T) {
	t.Run("should return all regions successfully with default pagination values", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockRegionService := serviceMock.NewMockRegionService(t)
		handler := region.GetAllRegions(mockRegionService)

		expectedRegions := []*entities.Region{
			{ID: 1, Name: "Region A"},
			{ID: 2, Name: "Region B"},
		}

		mockRegionService.EXPECT().GetAll(
			mock.Anything,
		).Return(expectedRegions, int64(len(expectedRegions)), nil)

		app.Get("/v1/region", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/region", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.RegionListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Len(t, response.Data, 2)
		assert.Equal(t, "Region A", response.Data[0].Name)
		assert.Equal(t, "Region B", response.Data[1].Name)

		// assert pagination
		assert.Empty(t, response.Pagination)

		mockRegionService.AssertExpectations(t)
	})

	t.Run("should return all regions successfully with limit 1 and offset 1", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockRegionService := serviceMock.NewMockRegionService(t)
		handler := region.GetAllRegions(mockRegionService)

		expectedRegions := []*entities.Region{
			{ID: 2, Name: "Region B"},
		}

		mockRegionService.EXPECT().GetAll(
			mock.Anything,
		).Return(expectedRegions, int64(len(expectedRegions)), nil)

		app.Get("/v1/region", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/region?page=2&limit=1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.RegionListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Len(t, response.Data, 1)
		assert.Equal(t, "Region B", response.Data[0].Name)

		// assert pagination
		assert.Equal(t, int32(2), response.Pagination.CurrentPage)
		assert.Equal(t, int64(1), response.Pagination.Total)
		assert.Nil(t, response.Pagination.NextPage)
		assert.Equal(t, int32(1), *response.Pagination.PrevPage)
		assert.Equal(t, int32((len(expectedRegions))/1), response.Pagination.TotalPages)

		mockRegionService.AssertExpectations(t)
	})

	t.Run("should return error when page is invalid", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockRegionService := serviceMock.NewMockRegionService(t)
		handler := region.GetAllRegions(mockRegionService)
		app.Get("/v1/region", handler)

		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/region?page=0&limit=1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockRegionService.AssertExpectations(t)
	})

	t.Run("should return error when limit is invalid", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockRegionService := serviceMock.NewMockRegionService(t)
		handler := region.GetAllRegions(mockRegionService)
		app.Get("/v1/region", handler)

		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/region?page=1&limit=0", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockRegionService.AssertExpectations(t)
	})

	t.Run("should return empty region list when no regions found", func(t *testing.T) {
		mockRegionService := serviceMock.NewMockRegionService(t)
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		handler := region.GetAllRegions(mockRegionService)

		mockRegionService.EXPECT().GetAll(
			mock.Anything,
		).Return([]*entities.Region{}, int64(0), nil)

		app.Get("/v1/region", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/region", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.RegionListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Len(t, response.Data, 0)

		// assert pagination
		assert.Empty(t, response.Pagination)

		mockRegionService.AssertExpectations(t)
	})

	t.Run("should return 500 when service returns an error", func(t *testing.T) {
		mockRegionService := serviceMock.NewMockRegionService(t)
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		handler := region.GetAllRegions(mockRegionService)

		mockRegionService.EXPECT().GetAll(
			mock.Anything,
		).Return(nil, int64(0), errors.New("service error"))

		app.Get("/v1/region", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/region", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockRegionService.AssertExpectations(t)
	})
}

func TestGetRegionByID(t *testing.T) {
	t.Run("should return region successfully", func(t *testing.T) {
		mockRegionService := serviceMock.NewMockRegionService(t)
		app := fiber.New()
		handler := region.GetRegionByID(mockRegionService)

		expectedRegion := &entities.Region{
			ID:   1,
			Name: "Region A",
		}

		mockRegionService.EXPECT().GetByID(
			mock.Anything,
			int32(1),
		).Return(expectedRegion, nil)

		app.Get("/v1/region/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/region/1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response entities.Region
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)
		assert.Equal(t, "Region A", response.Name)

		mockRegionService.AssertExpectations(t)
	})

	t.Run("should return 400 for invalid ID", func(t *testing.T) {
		mockRegionService := serviceMock.NewMockRegionService(t)
		app := fiber.New()
		handler := region.GetRegionByID(mockRegionService)

		app.Get("/v1/region/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/region/invalid-id", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockRegionService.AssertNotCalled(t, "GetByID", mock.Anything, mock.Anything)
	})

	t.Run("should return 404 when region not found", func(t *testing.T) {
		mockRegionService := serviceMock.NewMockRegionService(t)
		app := fiber.New()
		handler := region.GetRegionByID(mockRegionService)

		mockRegionService.EXPECT().GetByID(
			mock.Anything,
			int32(1),
		).Return(nil, service.NewError(service.NotFound, "region not found"))

		app.Get("/v1/region/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/region/1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)

		mockRegionService.AssertExpectations(t)
	})

	t.Run("should return 500 when service returns an error", func(t *testing.T) {
		mockRegionService := serviceMock.NewMockRegionService(t)
		app := fiber.New()
		handler := region.GetRegionByID(mockRegionService)

		mockRegionService.EXPECT().GetByID(
			mock.Anything,
			int32(1),
		).Return(nil, errors.New("service error"))

		app.Get("/v1/region/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/region/1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockRegionService.AssertExpectations(t)
	})
}
