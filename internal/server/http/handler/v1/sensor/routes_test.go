package sensor_test

import (
	"context"
	"net/http"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/sensor"
	"github.com/green-ecolution/backend/internal/server/http/middleware"
	serviceMock "github.com/green-ecolution/backend/internal/service/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestRegisterRoutes(t *testing.T) {
	t.Run("/v1/sensor", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockSensorService := serviceMock.NewMockSensorService(t)
			app := fiber.New()
			app.Use(middleware.PaginationMiddleware())
			sensor.RegisterRoutes(app, mockSensorService)

			ctx := context.WithValue(context.Background(), "page", int32(1))
			ctx = context.WithValue(ctx, "limit", int32(-1))

			mockSensorService.EXPECT().GetAll(
				mock.Anything,
				entities.Query{},
			).Return(TestSensorList, int64(len(TestSensorList)), nil)

			// when
			req, _ := http.NewRequestWithContext(ctx, http.MethodGet, "/", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})
	})

	t.Run("/v1/sensor/data/:id", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockSensorService := serviceMock.NewMockSensorService(t)
			app := fiber.New()
			sensor.RegisterRoutes(app, mockSensorService)

			mockSensorService.EXPECT().GetAllDataByID(
				mock.Anything,
				"sensor-1",
			).Return([]*entities.SensorData{TestSensorData}, nil)

			// when
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/data/sensor-1", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})
	})

	t.Run("/v1/sensor/:id", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockSensorService := serviceMock.NewMockSensorService(t)
			app := fiber.New()
			sensor.RegisterRoutes(app, mockSensorService)

			mockSensorService.EXPECT().GetByID(
				mock.Anything,
				"sensor-1",
			).Return(TestSensor, nil)

			// when
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/sensor-1", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})

		t.Run("should call DELETE handler", func(t *testing.T) {
			mockSensorService := serviceMock.NewMockSensorService(t)
			app := fiber.New()
			sensor.RegisterRoutes(app, mockSensorService)

			mockSensorService.EXPECT().Delete(
				mock.Anything,
				"sensor-1",
			).Return(nil)

			// when
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodDelete, "/sensor-1", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusNoContent, resp.StatusCode)
		})
	})
}
