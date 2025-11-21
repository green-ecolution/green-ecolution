package vehicle_test

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/vehicle"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/middleware"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/service/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestGetAllVehicles(t *testing.T) {
	t.Run("should return all vehicles successfully with default pagination values", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetAllVehicles(mockVehicleService)
		app.Get("/v1/vehicle", handler)

		mockVehicleService.EXPECT().GetAll(
			mock.Anything,
			entities.VehicleQuery{},
		).Return(TestVehicles, int64(len(TestVehicles)), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.VehicleListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Equal(t, 2, len(response.Data))
		assert.Equal(t, TestVehicles[0].ID, response.Data[0].ID)

		// assert pagination
		assert.Empty(t, response.Pagination)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return all vehicles successfully with limit 1 and offset 0", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetAllVehicles(mockVehicleService)
		app.Get("/v1/vehicle", handler)

		mockVehicleService.EXPECT().GetAll(
			mock.Anything,
			entities.VehicleQuery{},
		).Return(TestVehicles, int64(len(TestVehicles)), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle?page=1&limit=1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.VehicleListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Equal(t, 2, len(response.Data))
		assert.Equal(t, TestVehicles[0].ID, response.Data[0].ID)

		// assert pagination
		assert.Equal(t, int32(1), response.Pagination.CurrentPage)
		assert.Equal(t, int64(len(TestVehicles)), response.Pagination.Total)
		assert.Equal(t, int32(2), *response.Pagination.NextPage)
		assert.Empty(t, response.Pagination.PrevPage)
		assert.Equal(t, int32((len(TestVehicles))/1), response.Pagination.TotalPages)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return error when page is invalid", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetAllVehicles(mockVehicleService)
		app.Get("/v1/vehicle", handler)

		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle?page=0&limit=1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return error when limit is invalid", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetAllVehicles(mockVehicleService)
		app.Get("/v1/vehicle", handler)

		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle?page=1&limit=0", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return all vehicles successfully with provider", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetAllVehicles(mockVehicleService)
		app.Get("/v1/vehicle", handler)

		mockVehicleService.EXPECT().GetAll(
			mock.Anything,
			entities.VehicleQuery{Query: entities.Query{Provider: "test-provider"}},
		).Return(TestVehicles, int64(0), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle", nil)
		query := req.URL.Query()
		query.Add("provider", "test-provider")
		req.URL.RawQuery = query.Encode()
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.VehicleListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)
		assert.Equal(t, 2, len(response.Data))
		assert.Equal(t, TestVehicles[0].ID, response.Data[0].ID)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return all vehicles by one type successfully with default pagination values", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetAllVehicles(mockVehicleService)
		app.Get("/v1/vehicle", handler)

		mockVehicleService.EXPECT().GetAll(
			mock.Anything,
			entities.VehicleQuery{Type: "transporter"},
		).Return([]*entities.Vehicle{TestVehicles[1]}, int64(1), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle?type=transporter", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.VehicleListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Equal(t, 1, len(response.Data))

		// assert pagination
		assert.Empty(t, response.Pagination)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return all vehicles by one type successfully with limit 1 and offset 0", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetAllVehicles(mockVehicleService)
		app.Get("/v1/vehicle", handler)

		mockVehicleService.EXPECT().GetAll(
			mock.Anything, entities.VehicleQuery{Type: "transporter"},
		).Return([]*entities.Vehicle{TestVehicles[1]}, int64(1), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle?type=transporter&page=1&limit=1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.VehicleListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Equal(t, 1, len(response.Data))

		// assert pagination
		assert.Equal(t, int32(1), response.Pagination.CurrentPage)
		assert.Equal(t, int64(1), response.Pagination.Total)
		assert.Empty(t, response.Pagination.NextPage)
		assert.Empty(t, response.Pagination.PrevPage)
		assert.Equal(t, int32(1), response.Pagination.TotalPages)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return an empty list when no vehicles are available", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetAllVehicles(mockVehicleService)
		app.Get("/v1/vehicle", handler)

		mockVehicleService.EXPECT().GetAll(
			mock.Anything, entities.VehicleQuery{},
		).Return([]*entities.Vehicle{}, int64(0), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.VehicleListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)
		assert.NoError(t, err)

		// assert data
		assert.Equal(t, 0, len(response.Data))

		// assert pagination
		assert.Empty(t, response.Pagination)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return 500 Internal Server Error when service fails", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetAllVehicles(mockVehicleService)
		app.Get("/v1/vehicle", handler)

		mockVehicleService.EXPECT().GetAll(
			mock.Anything, entities.VehicleQuery{},
		).Return(nil, int64(0), fiber.NewError(fiber.StatusInternalServerError, "service error"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return error when service fails due to invalid type parameter", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetAllVehicles(mockVehicleService)
		app.Get("/v1/vehicle", handler)

		mockVehicleService.EXPECT().GetAll(
			mock.Anything, entities.VehicleQuery{Type: "invalid"},
		).Return(nil, int64(0), fiber.NewError(fiber.ErrBadRequest.Code, "service error"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle?type=invalid", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})
}

func TestGetVehicleByID(t *testing.T) {
	t.Run("should return vehicle successfully", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetVehicleByID(mockVehicleService)
		app.Get("/v1/vehicle/:id", handler)

		mockVehicleService.EXPECT().GetByID(
			mock.Anything,
			int32(1),
		).Return(TestVehicle, nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle/1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.VehicleResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)
		assert.Equal(t, TestVehicle.ID, response.ID)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return 400 Bad Request for invalid vehicle ID", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetVehicleByID(mockVehicleService)
		app.Get("/v1/vehicle/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle/invalid-id", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})

	t.Run("should return 404 Not Found if vehicle does not exist", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetVehicleByID(mockVehicleService)
		app.Get("/v1/vehicle/:id", handler)

		mockVehicleService.EXPECT().GetByID(
			mock.Anything,
			int32(999),
		).Return(nil, service.NewError(service.NotFound, "not found"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle/999", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return 500 Internal Server Error for service failure", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetVehicleByID(mockVehicleService)
		app.Get("/v1/vehicle/:id", handler)

		mockVehicleService.EXPECT().GetByID(
			mock.Anything,
			int32(1),
		).Return(nil, fiber.NewError(fiber.StatusInternalServerError, "service error"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle/1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})
}

func TestGetVehicleByPlate(t *testing.T) {
	t.Run("should return vehicle successfully", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetVehicleByPlate(mockVehicleService)
		app.Get("/v1/vehicle/plate/:plate", handler)
		plate := "FL%20TBZ%201234"

		mockVehicleService.EXPECT().GetByPlate(
			mock.Anything,
			plate,
		).Return(TestVehicle, nil)

		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle/plate/"+plate, nil)

		resp, err := app.Test(req, -1)
		assert.NoError(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		defer resp.Body.Close()

		var response serverEntities.VehicleResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)
		assert.Equal(t, TestVehicle.ID, response.ID)
		assert.Equal(t, TestVehicle.NumberPlate, response.NumberPlate)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return 400 Bad Request for invalid vehicle plate", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetVehicleByID(mockVehicleService)
		app.Get("/v1/vehicle/plate/:plate", handler)
		plate := "%20"

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle/plate/"+plate, nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})

	t.Run("should return 404 Not Found if vehicle does not exist", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetVehicleByPlate(mockVehicleService)
		app.Get("/v1/vehicle/plate/:plate", handler)
		plate := "FL%20TBZ%201244"

		mockVehicleService.EXPECT().GetByPlate(
			mock.Anything,
			plate,
		).Return(nil, service.NewError(service.NotFound, "not found"))

		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle/plate/"+plate, nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return 500 Internal Server Error for service failure", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.GetVehicleByPlate(mockVehicleService)
		app.Get("/v1/vehicle/plate/:plate", handler)
		plate := "FL%20TBZ%201244"

		mockVehicleService.EXPECT().GetByPlate(
			mock.Anything,
			plate,
		).Return(nil, fiber.NewError(fiber.StatusInternalServerError, "service error"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/vehicle/plate/"+plate, nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})
}

func TestCreateVehicle(t *testing.T) {
	t.Run("should create vehicle successfully", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.CreateVehicle(mockVehicleService)
		app.Post("/v1/vehicle", handler)

		mockVehicleService.EXPECT().Create(
			mock.Anything,
			mock.AnythingOfType("*entities.VehicleCreate"),
		).Return(TestVehicle, nil)

		// when
		body, _ := json.Marshal(TestVehicleRequest)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/vehicle", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusCreated, resp.StatusCode)

		var response serverEntities.VehicleResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, TestVehicleRequest.NumberPlate, response.NumberPlate)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return 400 Bad Request for invalid request body", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.CreateVehicle(mockVehicleService)
		app.Post("/v1/vehicle", handler)

		invalidRequestBody := []byte(`{"invalid_field": "value"}`)

		body, _ := json.Marshal(invalidRequestBody)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/vehicle", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})

	t.Run("should return 500 Internal Server Error for service failure", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.CreateVehicle(mockVehicleService)
		app.Post("/v1/vehicle", handler)

		mockVehicleService.EXPECT().Create(
			mock.Anything,
			mock.AnythingOfType("*entities.VehicleCreate"),
		).Return(nil, fiber.NewError(fiber.StatusInternalServerError, "service error"))

		// when
		body, _ := json.Marshal(TestVehicleRequest)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/vehicle", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})
}

func TestUpdateVehicle(t *testing.T) {
	t.Run("should update vehicle successfully", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.UpdateVehicle(mockVehicleService)
		app.Put("/v1/vehicle/:id", handler)

		mockVehicleService.EXPECT().Update(
			mock.Anything,
			int32(1),
			mock.Anything,
		).Return(TestVehicle, nil)

		// when
		body, _ := json.Marshal(TestVehicleRequest)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/v1/vehicle/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.VehicleResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, TestVehicleRequest.NumberPlate, response.NumberPlate)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return 400 Bad Request for invalid vehicle ID", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.UpdateVehicle(mockVehicleService)
		app.Put("/v1/vehicle/:id", handler)

		// when
		body, _ := json.Marshal(TestVehicleRequest)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/v1/vehicle/invalid-id", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})

	t.Run("should return 400 Bad Request for invalid request body", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.UpdateVehicle(mockVehicleService)
		app.Put("/v1/vehicle/:id", handler)

		invalidRequestBody := []byte(`{"invalid_field": "value"}`)

		// when
		body, _ := json.Marshal(invalidRequestBody)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/v1/vehicle/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})

	t.Run("should return 404 Not Found if cluster does not exist", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.UpdateVehicle(mockVehicleService)
		app.Put("/v1/vehicle/:id", handler)

		mockVehicleService.EXPECT().Update(
			mock.Anything,
			int32(1),
			mock.Anything,
		).Return(nil, service.NewError(service.NotFound, "not found"))

		// when
		body, _ := json.Marshal(TestVehicleRequest)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/v1/vehicle/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return 500 Internal Server Error for service failure", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.UpdateVehicle(mockVehicleService)
		app.Put("/v1/vehicle/:id", handler)

		mockVehicleService.EXPECT().Update(
			mock.Anything,
			int32(1),
			mock.Anything,
		).Return(nil, fiber.NewError(fiber.StatusInternalServerError, "service error"))

		// when
		body, _ := json.Marshal(TestVehicleRequest)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/v1/vehicle/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})
}

func TestDeleteVehicle(t *testing.T) {
	t.Run("should delete vehicle successfully", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.DeleteVehicle(mockVehicleService)
		app.Delete("/v1/vehicle/:id", handler)

		clusterID := 1
		mockVehicleService.EXPECT().Delete(
			mock.Anything,
			int32(clusterID),
		).Return(nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodDelete, "/v1/vehicle/1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNoContent, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return 400 for invalid ID format", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.DeleteVehicle(mockVehicleService)
		app.Delete("/v1/vehicle/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodDelete, "/v1/vehicle/invalid-id", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})

	t.Run("should return 404 for non-existing vehicle", func(t *testing.T) {
		app := fiber.New()
		mockVehicleService := serviceMock.NewMockVehicleService(t)
		handler := vehicle.DeleteVehicle(mockVehicleService)
		app.Delete("/v1/vehicle/:id", handler)

		clusterID := 999
		mockVehicleService.EXPECT().Delete(
			mock.Anything,
			int32(clusterID),
		).Return(service.NewError(service.NotFound, "not found"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodDelete, "/v1/vehicle/999", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)

		mockVehicleService.AssertExpectations(t)
	})
}
