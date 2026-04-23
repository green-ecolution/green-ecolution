package wateringplan_test

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"strconv"
	"testing"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/application/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	wateringplan "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/handler/v1/watering_plan"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/middleware"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func TestGetAllWateringPlans(t *testing.T) {
	t.Run("should return all watering plans successfully with default pagination values", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.GetAllWateringPlans(mockWateringPlanService)
		app.Get("/v1/watering-plan", handler)

		mockWateringPlanService.EXPECT().GetAll(
			mock.Anything,
			entities.Query{},
		).Return(TestWateringPlans, int64(len(TestWateringPlans)), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/watering-plan", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.WateringPlanListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Equal(t, len(TestWateringPlans), len(response.Data))
		assert.Equal(t, TestWateringPlans[0].Date, response.Data[0].Date)

		// assert pagination
		assert.Empty(t, response.Pagination)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return all watering plans successfully with limit 1 and offset 0", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.GetAllWateringPlans(mockWateringPlanService)
		app.Get("/v1/watering-plan", handler)

		mockWateringPlanService.EXPECT().GetAll(
			mock.Anything,
			entities.Query{},
		).Return(TestWateringPlans, int64(len(TestWateringPlans)), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/watering-plan?page=1&limit=1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.WateringPlanListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Equal(t, len(TestWateringPlans), len(response.Data))
		assert.Equal(t, TestWateringPlans[0].Date, response.Data[0].Date)

		// assert pagination
		assert.Equal(t, int32(1), response.Pagination.CurrentPage)
		assert.Equal(t, int64(len(TestWateringPlans)), response.Pagination.Total)
		assert.Equal(t, int32(2), *response.Pagination.NextPage)
		assert.Empty(t, response.Pagination.PrevPage)
		assert.Equal(t, int32((len(TestWateringPlans))/1), response.Pagination.TotalPages)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return error when page is invalid", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.GetAllWateringPlans(mockWateringPlanService)
		app.Get("/v1/watering-plan", handler)

		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/watering-plan?page=0&limit=1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return error when limit is invalid", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.GetAllWateringPlans(mockWateringPlanService)
		app.Get("/v1/watering-plan", handler)

		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/watering-plan?page=1&limit=0", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return all watering plans successfully with provider", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.GetAllWateringPlans(mockWateringPlanService)
		app.Get("/v1/watering-plan", handler)

		mockWateringPlanService.EXPECT().GetAll(
			mock.Anything,
			entities.Query{Provider: "test-provider"},
		).Return(TestWateringPlans, int64(len(TestWateringPlans)), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/watering-plan", nil)
		query := req.URL.Query()
		query.Add("provider", "test-provider")
		req.URL.RawQuery = query.Encode()
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.WateringPlanListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)
		assert.Equal(t, len(TestWateringPlans), len(response.Data))
		assert.Equal(t, TestWateringPlans[0].Date, response.Data[0].Date)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return an empty list when no watering plans are available", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.GetAllWateringPlans(mockWateringPlanService)
		app.Get("/v1/watering-plan", handler)

		mockWateringPlanService.EXPECT().GetAll(
			mock.Anything,
			entities.Query{},
		).Return([]*entities.WateringPlan{}, int64(0), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/watering-plan", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.WateringPlanListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Equal(t, 0, len(response.Data))

		// assert pagination
		assert.Empty(t, response.Pagination)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return 500 Internal Server Error when service fails", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.GetAllWateringPlans(mockWateringPlanService)
		app.Get("/v1/watering-plan", handler)

		mockWateringPlanService.EXPECT().GetAll(
			mock.Anything,
			entities.Query{},
		).Return(nil, int64(0), fiber.NewError(fiber.StatusInternalServerError, "service error"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/watering-plan", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockWateringPlanService.AssertExpectations(t)
	})
}

func TestGetWateringPlanByID(t *testing.T) {
	t.Run("should return watering plan successfully", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.GetWateringPlanByID(mockWateringPlanService)
		app.Get("/v1/watering-plan/:id", handler)

		mockWateringPlanService.EXPECT().GetByID(
			mock.Anything,
			int32(1),
		).Return(TestWateringPlans[0], nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/watering-plan/1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.WateringPlanResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)
		assert.Equal(t, TestWateringPlans[0].Date, response.Date)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return 400 Bad Request for invalid watering plan ID", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.GetWateringPlanByID(mockWateringPlanService)
		app.Get("/v1/watering-plan/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/watering-plan/invalid", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})

	t.Run("should return 404 Not Found if watering plan does not exist", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.GetWateringPlanByID(mockWateringPlanService)
		app.Get("/v1/watering-plan/:id", handler)

		mockWateringPlanService.EXPECT().GetByID(
			mock.Anything,
			int32(999),
		).Return(nil, ports.NewError(ports.NotFound, "not found"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/watering-plan/999", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return 500 Internal Server Error for service failure", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.GetWateringPlanByID(mockWateringPlanService)
		app.Get("/v1/watering-plan/:id", handler)

		mockWateringPlanService.EXPECT().GetByID(
			mock.Anything,
			int32(1),
		).Return(nil, fiber.NewError(fiber.StatusInternalServerError, "service error"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/watering-plan/1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockWateringPlanService.AssertExpectations(t)
	})
}

func TestCreateWateringPlan(t *testing.T) {
	t.Run("should create watering plan successfully", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.CreateWateringPlan(mockWateringPlanService)
		app.Post("/v1/watering-plan", handler)

		mockWateringPlanService.EXPECT().Create(
			mock.Anything,
			mock.AnythingOfType("*entities.WateringPlanCreate"),
		).Return(TestWateringPlans[0], nil)

		// when
		body, _ := json.Marshal(TestWateringPlanRequest)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/watering-plan", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusCreated, resp.StatusCode)

		var response serverEntities.WateringPlanResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, TestWateringPlans[0].Date, response.Date)
		assert.Equal(t, TestWateringPlans[0].Transporter.NumberPlate, response.Transporter.NumberPlate)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return 400 Bad Request for invalid request body", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.CreateWateringPlan(mockWateringPlanService)
		app.Post("/v1/watering-plan", handler)

		invalidRequestBody := []byte(`{"invalid_field": "value"}`)

		// when
		body, _ := json.Marshal(invalidRequestBody)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/watering-plan", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})

	t.Run("should return 500 Internal Server Error for service failure", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.CreateWateringPlan(mockWateringPlanService)
		app.Post("/v1/watering-plan", handler)

		mockWateringPlanService.EXPECT().Create(
			mock.Anything,
			mock.AnythingOfType("*entities.WateringPlanCreate"),
		).Return(nil, fiber.NewError(fiber.StatusInternalServerError, "service error"))

		// when
		body, _ := json.Marshal(TestWateringPlanRequest)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/watering-plan", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockWateringPlanService.AssertExpectations(t)
	})
}

func TestUpdateWateringPlan(t *testing.T) {
	t.Run("should update watering plan successfully", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.UpdateWateringPlan(mockWateringPlanService)
		app.Put("/v1/watering-plan/:id", handler)

		mockWateringPlanService.EXPECT().Update(
			mock.Anything,
			int32(1),
			mock.Anything,
		).Return(TestWateringPlans[0], nil)

		// when
		body, _ := json.Marshal(TestWateringPlanUpdateRequest)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/v1/watering-plan/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.WateringPlanResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return 400 Bad Request for invalid watering plan ID", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.UpdateWateringPlan(mockWateringPlanService)
		app.Put("/v1/watering-plan/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/v1/watering-plan/invalid", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})

	t.Run("should return 400 Bad Request for invalid request body", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.UpdateWateringPlan(mockWateringPlanService)
		app.Put("/v1/watering-plan/:id", handler)

		invalidRequestBody := []byte(`{"invalid_field": "value"}`)

		// when
		body, _ := json.Marshal(invalidRequestBody)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/v1/watering-plan/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})

	t.Run("should return 404 Not Found if watering plan does not exist", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.UpdateWateringPlan(mockWateringPlanService)
		app.Put("/v1/watering-plan/:id", handler)

		mockWateringPlanService.EXPECT().Update(
			mock.Anything,
			int32(1),
			mock.Anything,
		).Return(nil, ports.NewError(ports.NotFound, "not found"))

		// when
		body, _ := json.Marshal(TestWateringPlanUpdateRequest)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/v1/watering-plan/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return 500 Internal Server Error for service failure", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.UpdateWateringPlan(mockWateringPlanService)
		app.Put("/v1/watering-plan/:id", handler)

		mockWateringPlanService.EXPECT().Update(mock.Anything, int32(1), mock.Anything).Return(nil, fiber.NewError(fiber.StatusInternalServerError, "service error"))

		// when
		body, _ := json.Marshal(TestWateringPlanUpdateRequest)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/v1/watering-plan/1", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

	})
}

func TestDeleteWateringPlan(t *testing.T) {
	t.Run("should delete watering plan successfully", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.DeleteWateringPlan(mockWateringPlanService)
		app.Delete("/v1/watering-plan/:id", handler)

		wateringPlanID := 1
		mockWateringPlanService.EXPECT().Delete(mock.Anything, int32(wateringPlanID)).Return(nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodDelete, "/v1/watering-plan/"+strconv.Itoa(wateringPlanID), nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNoContent, resp.StatusCode)

		mockWateringPlanService.AssertExpectations(t)
	})

	t.Run("should return 400 for invalid ID format", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.DeleteWateringPlan(mockWateringPlanService)
		app.Delete("/v1/watering-plan/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodDelete, "/v1/watering-plan/invalid_id", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})

	t.Run("should return 404 for non-existing tree cluster", func(t *testing.T) {
		app := fiber.New()
		mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
		handler := wateringplan.DeleteWateringPlan(mockWateringPlanService)
		app.Delete("/v1/watering-plan/:id", handler)

		wateringPlanID := 999
		mockWateringPlanService.EXPECT().Delete(
			mock.Anything,
			int32(wateringPlanID),
		).Return(ports.NewError(ports.NotFound, "not found"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodDelete, "/v1/watering-plan/"+strconv.Itoa(wateringPlanID), nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)
		mockWateringPlanService.AssertExpectations(t)
	})
}

func TestCreateWateringPlanValidation(t *testing.T) {
	tomorrow := time.Now().Add(24 * time.Hour).Truncate(24 * time.Hour)
	yesterday := time.Now().Add(-24 * time.Hour).Truncate(24 * time.Hour)

	base := func() serverEntities.WateringPlanCreateRequest {
		return serverEntities.WateringPlanCreateRequest{
			Date:           tomorrow,
			TreeClusterIDs: []*int32{utils.P(int32(1))},
			TransporterID:  utils.P(int32(1)),
			UserIDs:        []string{"6a1078e8-80fd-458f-b74e-e388fe2dd6ab"},
		}
	}

	cases := []struct {
		name   string
		mutate func(r *serverEntities.WateringPlanCreateRequest)
	}{
		{"date in the past", func(r *serverEntities.WateringPlanCreateRequest) { r.Date = yesterday }},
		{"missing tree cluster ids", func(r *serverEntities.WateringPlanCreateRequest) { r.TreeClusterIDs = nil }},
		{"empty tree cluster ids", func(r *serverEntities.WateringPlanCreateRequest) { r.TreeClusterIDs = []*int32{} }},
		{"missing transporter id", func(r *serverEntities.WateringPlanCreateRequest) { r.TransporterID = nil }},
		{"missing user ids", func(r *serverEntities.WateringPlanCreateRequest) { r.UserIDs = nil }},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			app := fiber.New()
			mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
			app.Post("/v1/watering-plan", wateringplan.CreateWateringPlan(mockWateringPlanService))

			req := base()
			tc.mutate(&req)
			body, _ := json.Marshal(req)
			httpReq, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/watering-plan", bytes.NewBuffer(body))
			httpReq.Header.Set("Content-Type", "application/json")
			resp, err := app.Test(httpReq, -1)
			defer resp.Body.Close()

			assert.Nil(t, err)
			assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
		})
	}
}

func TestUpdateWateringPlanValidation(t *testing.T) {
	tomorrow := time.Now().Add(24 * time.Hour).Truncate(24 * time.Hour)

	base := func() serverEntities.WateringPlanUpdateRequest {
		return serverEntities.WateringPlanUpdateRequest{
			Date:           tomorrow,
			TreeClusterIDs: []*int32{utils.P(int32(1))},
			TransporterID:  utils.P(int32(1)),
			UserIDs:        []string{"6a1078e8-80fd-458f-b74e-e388fe2dd6ab"},
			Status:         serverEntities.WateringPlanStatusPlanned,
		}
	}

	cases := []struct {
		name   string
		mutate func(r *serverEntities.WateringPlanUpdateRequest)
	}{
		{"invalid status enum", func(r *serverEntities.WateringPlanUpdateRequest) { r.Status = "bogus" }},
		{"cancellation note without canceled status", func(r *serverEntities.WateringPlanUpdateRequest) {
			r.Status = serverEntities.WateringPlanStatusPlanned
			r.CancellationNote = "oops"
		}},
		{"evaluation without finished status", func(r *serverEntities.WateringPlanUpdateRequest) {
			r.Status = serverEntities.WateringPlanStatusPlanned
			r.Evaluation = []*serverEntities.EvaluationValue{{TreeClusterID: int32(1), ConsumedWater: utils.P(100.0)}}
		}},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			app := fiber.New()
			mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
			app.Put("/v1/watering-plan/:id", wateringplan.UpdateWateringPlan(mockWateringPlanService))

			req := base()
			tc.mutate(&req)
			body, _ := json.Marshal(req)
			httpReq, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/v1/watering-plan/1", bytes.NewBuffer(body))
			httpReq.Header.Set("Content-Type", "application/json")
			resp, err := app.Test(httpReq, -1)
			defer resp.Body.Close()

			assert.Nil(t, err)
			assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
		})
	}
}
