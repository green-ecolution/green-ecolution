package wateringplan_test

import (
	"bytes"
	"context"
	"encoding/json"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"net/http"
	"testing"

	"github.com/gofiber/fiber/v2"
	wateringplan "github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/watering_plan"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/middleware"
	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/service/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestRegisterRoutes(t *testing.T) {
	t.Run("/v1/watering-plan", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
			app := fiber.New()
			app.Use(middleware.PaginationMiddleware())
			wateringplan.RegisterRoutes(app, mockWateringPlanService)

			ctx := context.WithValue(context.Background(), "page", int32(1))
			ctx = context.WithValue(ctx, "limit", int32(-1))

			mockWateringPlanService.EXPECT().GetAll(
				mock.Anything,
				entities.Query{},
			).Return(TestWateringPlans, int64(len(TestWateringPlans)), nil)

			// when
			req, _ := http.NewRequestWithContext(ctx, http.MethodGet, "/", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})

		t.Run("should call POST handler", func(t *testing.T) {
			mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
			app := fiber.New()
			wateringplan.RegisterRoutes(app, mockWateringPlanService)

			mockWateringPlanService.EXPECT().Create(
				mock.Anything,
				mock.AnythingOfType("*entities.WateringPlanCreate"),
			).Return(TestWateringPlans[0], nil)

			// when
			body, _ := json.Marshal(TestWateringPlanRequest)
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/", bytes.NewBuffer(body))
			req.Header.Set("Content-Type", "application/json")

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusCreated, resp.StatusCode)
		})
	})

	t.Run("/v1/watering-plan/:id", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
			app := fiber.New()
			wateringplan.RegisterRoutes(app, mockWateringPlanService)

			mockWateringPlanService.EXPECT().GetByID(
				mock.Anything,
				int32(1),
			).Return(TestWateringPlans[0], nil)

			// when
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/1", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})

		t.Run("should call PUT handler", func(t *testing.T) {
			mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
			app := fiber.New()
			wateringplan.RegisterRoutes(app, mockWateringPlanService)

			mockWateringPlanService.EXPECT().Update(
				mock.Anything,
				int32(1),
				mock.Anything,
			).Return(TestWateringPlans[0], nil)

			// when
			body, _ := json.Marshal(TestWateringPlanUpdateRequest)
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/1", bytes.NewBuffer(body))
			req.Header.Set("Content-Type", "application/json")

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})

		t.Run("should call DELETE handler", func(t *testing.T) {
			mockWateringPlanService := serviceMock.NewMockWateringPlanService(t)
			app := fiber.New()
			wateringplan.RegisterRoutes(app, mockWateringPlanService)

			mockWateringPlanService.EXPECT().Delete(
				mock.Anything,
				int32(1),
			).Return(nil)

			// when
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodDelete, "/1", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusNoContent, resp.StatusCode)
		})
	})
}
