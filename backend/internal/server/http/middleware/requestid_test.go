package middleware

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/stretchr/testify/assert"
)

func TestRequestID(t *testing.T) {
	t.Run("should add X-Request-ID header to each request", func(t *testing.T) {
		app := fiber.New()
		app.Use(RequestID())

		req := httptest.NewRequest(http.MethodGet, "/", nil)
		resp, err := app.Test(req, -1)

		assert.NoError(t, err)
		defer resp.Body.Close()
		assert.NotEmpty(t, resp.Header.Get("X-Request-ID"))
	})

	t.Run("should generate unique IDs for different requests", func(t *testing.T) {
		app := fiber.New()
		app.Use(RequestID())

		req1 := httptest.NewRequest(http.MethodGet, "/", nil)
		resp1, err1 := app.Test(req1, -1)
		assert.NoError(t, err1)

		defer resp1.Body.Close()
		id1 := resp1.Header.Get("X-Request-ID")
		assert.NotEmpty(t, id1)

		req2 := httptest.NewRequest(http.MethodGet, "/", nil)
		resp2, err2 := app.Test(req2, -1)
		assert.NoError(t, err2)

		defer resp2.Body.Close()
		id2 := resp2.Header.Get("X-Request-ID")
		assert.NotEmpty(t, id2)

		assert.NotEqual(t, id1, id2)
	})

	t.Run("should add X-Request-ID header to each request", func(t *testing.T) {
		app := fiber.New()
		app.Use(RequestID())

		req := httptest.NewRequest(http.MethodGet, "/", nil)
		resp, err := app.Test(req, -1)

		assert.NoError(t, err)
		defer resp.Body.Close()
		assert.NotEmpty(t, resp.Header.Get("X-Request-ID"))
	})

	t.Run("should generate unique IDs for different requests", func(t *testing.T) {
		app := fiber.New()
		app.Use(RequestID())

		req1 := httptest.NewRequest(http.MethodGet, "/", nil)
		resp1, err1 := app.Test(req1, -1)
		assert.NoError(t, err1)
		defer resp1.Body.Close()
		id1 := resp1.Header.Get("X-Request-ID")
		assert.NotEmpty(t, id1)

		req2 := httptest.NewRequest(http.MethodGet, "/", nil)
		resp2, err2 := app.Test(req2, -1)
		assert.NoError(t, err2)

		defer resp2.Body.Close()
		id2 := resp2.Header.Get("X-Request-ID")
		assert.NotEmpty(t, id2)

		assert.NotEqual(t, id1, id2)
	})

}
