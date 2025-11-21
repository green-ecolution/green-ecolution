package vehicle_test

import (
	"bytes"
	"context"
	"encoding/json"
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"net/http"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/vehicle"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/middleware"
	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/service/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestRegisterRoutes(t *testing.T) {
	t.Run("/v1/vehicle", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockVehicleService := serviceMock.NewMockVehicleService(t)
			app := fiber.New()
			app.Use(middleware.PaginationMiddleware())
			vehicle.RegisterRoutes(app, mockVehicleService)

			ctx := context.WithValue(context.Background(), "page", int32(1))
			ctx = context.WithValue(ctx, "limit", int32(-1))

			mockVehicleService.EXPECT().GetAll(
				mock.Anything, domain.VehicleQuery{},
			).Return(TestVehicles, int64(len(TestVehicles)), nil)

			// when
			req, _ := http.NewRequestWithContext(ctx, http.MethodGet, "/", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})

		t.Run("should call GET handler with vehicle type parameter", func(t *testing.T) {
			mockVehicleService := serviceMock.NewMockVehicleService(t)
			app := fiber.New()
			vehicle.RegisterRoutes(app, mockVehicleService)

			mockVehicleService.EXPECT().GetAll(
				mock.Anything, domain.VehicleQuery{Type: "transporter"},
			).Return(TestVehicles, int64(len(TestVehicles)), nil)

			// when
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/?type=transporter", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})

		t.Run("should call POST handler", func(t *testing.T) {
			mockVehicleService := serviceMock.NewMockVehicleService(t)
			app := fiber.New()
			vehicle.RegisterRoutes(app, mockVehicleService)

			mockVehicleService.EXPECT().Create(
				mock.Anything,
				mock.AnythingOfType("*entities.VehicleCreate"),
			).Return(TestVehicle, nil)

			// when
			body, _ := json.Marshal(TestVehicleRequest)
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/", bytes.NewBuffer(body))
			req.Header.Set("Content-Type", "application/json")

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusCreated, resp.StatusCode)
		})
	})

	t.Run("/v1/vehicle/:id", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockVehicleService := serviceMock.NewMockVehicleService(t)
			app := fiber.New()
			vehicle.RegisterRoutes(app, mockVehicleService)

			mockVehicleService.EXPECT().GetByID(
				mock.Anything,
				int32(1),
			).Return(TestVehicle, nil)

			// when
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/1", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})

		t.Run("should call PUT handler", func(t *testing.T) {
			mockVehicleService := serviceMock.NewMockVehicleService(t)
			app := fiber.New()
			vehicle.RegisterRoutes(app, mockVehicleService)

			mockVehicleService.EXPECT().Update(
				mock.Anything,
				int32(1),
				mock.Anything,
			).Return(TestVehicle, nil)

			// when
			body, _ := json.Marshal(TestVehicleRequest)
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/1", bytes.NewBuffer(body))
			req.Header.Set("Content-Type", "application/json")

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})

		t.Run("should call DELETE handler", func(t *testing.T) {
			mockVehicleService := serviceMock.NewMockVehicleService(t)
			app := fiber.New()
			vehicle.RegisterRoutes(app, mockVehicleService)

			mockVehicleService.EXPECT().Delete(
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
