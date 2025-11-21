package middleware

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gofiber/fiber/v2"
	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/service/_mock"
	"github.com/stretchr/testify/assert"
)

func TestHealthCheck(t *testing.T) {
	app := fiber.New()
	svc := serviceMock.NewMockServicesInterface(t)
	svc.EXPECT().AllServicesReady().Return(true)
	handler := HealthCheck(svc)
	app.Use(handler)

	t.Run("should return 200 OK for liveness probe", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/health", nil)
		resp, err := app.Test(req, -1)

		assert.NoError(t, err)
		defer resp.Body.Close()
		assert.Equal(t, http.StatusOK, resp.StatusCode)
	})

	t.Run("should return 200 OK for readiness probe when services are ready", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/ready", nil)
		resp, err := app.Test(req, -1)

		assert.NoError(t, err)
		defer resp.Body.Close()
		assert.Equal(t, http.StatusOK, resp.StatusCode)
	})

	t.Run("should return 404 for undefined endpoint", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/undefined", nil)
		resp, err := app.Test(req, -1)

		assert.NoError(t, err)
		defer resp.Body.Close()
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)
	})
	t.Run("should not return 200 for readiness probe when services are not ready", func(t *testing.T) {
		svc := serviceMock.NewMockServicesInterface(t)

		svc.EXPECT().AllServicesReady().Return(false).Once()

		app := fiber.New()
		handler := HealthCheck(svc)
		app.Use(handler)

		req := httptest.NewRequest(http.MethodGet, "/ready", nil)
		resp, err := app.Test(req, -1)

		assert.NoError(t, err)
		defer resp.Body.Close()
		assert.Equal(t, http.StatusServiceUnavailable, resp.StatusCode)
	})
}
