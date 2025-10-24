package middleware

import (
	"context"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/stretchr/testify/assert"
)

func TestHTTPLogger_EdgeCases(t *testing.T) {
	t.Run("should handle long processing time", func(t *testing.T) {
		app := fiber.New()
		app.Use(AppLogger(slog.Default))

		app.Get("/timeout", func(c *fiber.Ctx) error {

			ctx, cancel := context.WithTimeout(c.Context(), 5*time.Second)
			defer cancel()

			select {
			case <-time.After(6 * time.Second):
				return c.SendStatus(http.StatusOK)
			case <-ctx.Done():
				return c.SendStatus(http.StatusRequestTimeout)
			}
		})

		req := httptest.NewRequest(http.MethodGet, "/timeout", nil)
		resp, err := app.Test(req, -1)

		assert.NoError(t, err)
		defer resp.Body.Close()
		assert.Equal(t, http.StatusRequestTimeout, resp.StatusCode)
	})

	t.Run("should handle TRACE method request", func(t *testing.T) {
		app := fiber.New()
		app.Use(AppLogger(slog.Default))

		app.Trace("/trace", func(c *fiber.Ctx) error {
			return c.SendStatus(http.StatusOK)
		})

		req := httptest.NewRequest(http.MethodTrace, "/trace", nil)
		resp, err := app.Test(req, -1)

		assert.NoError(t, err)
		defer resp.Body.Close()
		assert.Equal(t, http.StatusOK, resp.StatusCode)
	})

	t.Run("should handle large number of simultaneous requests", func(t *testing.T) {
		app := fiber.New()
		app.Use(AppLogger(slog.Default))
		app.Get("/test", func(c *fiber.Ctx) error { return c.SendStatus(http.StatusOK) })

		concurrency := 100
		results := make(chan bool, concurrency)

		for i := 0; i < concurrency; i++ {
			go func() {
				req := httptest.NewRequest(http.MethodGet, "/test", nil)
				resp, err := app.Test(req, -1)
				results <- err == nil && resp.StatusCode == http.StatusOK
				defer resp.Body.Close()
			}()
		}

		for i := 0; i < concurrency; i++ {
			assert.True(t, <-results)
		}

	})

	t.Run("should handle request with unusual headers", func(t *testing.T) {
		app := fiber.New()
		app.Use(AppLogger(slog.Default))

		app.Get("/test", func(c *fiber.Ctx) error {
			return c.SendStatus(http.StatusOK)
		})

		req := httptest.NewRequest(http.MethodGet, "/test", nil)
		req.Header.Set("X-Unusual-Header", "UnusualHeaderValue")
		resp, err := app.Test(req, -1)

		assert.NoError(t, err)
		defer resp.Body.Close()
		assert.Equal(t, http.StatusOK, resp.StatusCode)
	})

	t.Run("should handle large URL path and query", func(t *testing.T) {
		app := fiber.New()
		app.Use(AppLogger(slog.Default))

		longPath := "/" + strings.Repeat("pathsegment/", 50)
		req := httptest.NewRequest(http.MethodGet, longPath, nil)
		resp, err := app.Test(req, -1)

		assert.NoError(t, err)
		defer resp.Body.Close()
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)
	})

	t.Run("should handle multilingual characters in path", func(t *testing.T) {
		app := fiber.New()
		app.Use(AppLogger(slog.Default))

		app.Get("/multilingual/こんにちは", func(c *fiber.Ctx) error {
			return c.SendStatus(http.StatusOK)
		})

		req := httptest.NewRequest(http.MethodGet, "/multilingual/こんにちは", nil)
		resp, err := app.Test(req, -1)

		assert.NoError(t, err)
		defer resp.Body.Close()
		assert.Equal(t, http.StatusOK, resp.StatusCode)
	})
}
