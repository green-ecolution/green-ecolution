package http

import (
	"bytes"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"strconv"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/middleware"
	"github.com/stretchr/testify/assert"
)

func TestHealthCheckMiddleware(t *testing.T) {
	t.Run("should return status 200 for health check", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.HealthCheck(nil))

		req := httptest.NewRequest("GET", "/health", nil)
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode, "Health check should return status 200")
	})

	t.Run("should handle health check with different HTTP methods", func(t *testing.T) {
		methods := []string{"POST", "PUT", "DELETE", "PATCH"}
		for _, method := range methods {
			t.Run("method: "+method, func(t *testing.T) {
				app := fiber.New()
				app.Use(middleware.HealthCheck(nil))

				req := httptest.NewRequest(method, "/health", nil)
				resp, err := app.Test(req, -1)
				if err != nil {
					t.Fatalf("Request failed: %v", err)
				}
				if resp != nil {
					defer resp.Body.Close()
				}

				assert.Nil(t, err)
				assert.Equal(t, http.StatusNotFound, resp.StatusCode, "Health check should only allow GET method")
			})
		}
	})

	t.Run("should return proper headers in response", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.HealthCheck(nil))

		req := httptest.NewRequest("GET", "/health", nil)
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode, "Health check should return status 200")
		assert.Equal(t, "text/plain; charset=utf-8", resp.Header.Get("Content-Type"), "Content-Type should be set to text/plain")
	})

	t.Run("should return correct response body", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.HealthCheck(nil))

		req := httptest.NewRequest("GET", "/health", nil)
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode, "Health check should return status 200")

		body, err := io.ReadAll(resp.Body)
		assert.Nil(t, err)
		assert.Equal(t, "OK", string(body), "Health check response body should be 'OK'")
	})
}

func TestHTTPLoggerMiddleware(t *testing.T) {

	t.Run("should log and return a response for valid GET request", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.AppLogger(slog.Default))

		app.Get("/log", func(c *fiber.Ctx) error {
			return c.SendString("Log endpoint")
		})

		req := httptest.NewRequest("GET", "/log", nil)
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode, "Response should return status 200")
	})

	t.Run("should return 404 for non-existent route with logging", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.AppLogger(slog.Default))

		req := httptest.NewRequest("GET", "/non-existent", nil)
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode, "Response should return status 404 for non-existent route")
	})

	t.Run("should handle logging for POST requests", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.AppLogger(slog.Default))

		app.Post("/log", func(c *fiber.Ctx) error {
			return c.SendString("Logged POST request")
		})

		req := httptest.NewRequest("POST", "/log", nil)
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode, "POST request should return status 200")
	})

	t.Run("should handle requests with headers and log them", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.AppLogger(slog.Default))

		app.Get("/log", func(c *fiber.Ctx) error {
			return c.SendString("Log endpoint with headers")
		})

		req := httptest.NewRequest("GET", "/log", nil)
		req.Header.Set("X-Custom-Header", "TestHeader")
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode, "Request with headers should return status 200")

	})

	t.Run("should handle large payloads and log request details", func(t *testing.T) {
		app := fiber.New(fiber.Config{
			BodyLimit: 20 * 1024 * 1024,
		})
		app.Use(middleware.AppLogger(slog.Default))

		app.Post("/log", func(c *fiber.Ctx) error {
			return c.SendString("Handled large payload")
		})

		largePayload := make([]byte, 10*1024*1024)
		req := httptest.NewRequest("POST", "/log", bytes.NewBuffer(largePayload))
		req.Header.Set("Content-Type", "application/json")

		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Equal(t, http.StatusOK, resp.StatusCode, "Large payload should be handled successfully")
	})

	t.Run("should handle 500 Internal Server Error response and log it", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.AppLogger(slog.Default))

		app.Get("/error", func(c *fiber.Ctx) error {
			return fiber.NewError(http.StatusInternalServerError, "Internal error")
		})

		req := httptest.NewRequest("GET", "/error", nil)
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode, "Internal server error should return status 500")
	})

	t.Run("should log requests with query parameters", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.AppLogger(slog.Default))

		app.Get("/log", func(c *fiber.Ctx) error {
			return c.SendString("Logged request with query parameters")
		})

		req := httptest.NewRequest("GET", "/log?param1=value1&param2=value2", nil)
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode, "Request with query parameters should return status 200")
	})

}

func TestRequestIDMiddleware(t *testing.T) {
	t.Run("should set X-Request-ID header", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.RequestID())
		app.Get("/request-id", func(c *fiber.Ctx) error {
			return c.SendStatus(http.StatusOK)
		})

		req := httptest.NewRequest("GET", "/request-id", nil)
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode, "RequestID middleware should allow the request to proceed")
		assert.NotEmpty(t, resp.Header.Get("X-Request-ID"), "X-Request-ID header should be set")
	})

}

func TestPublicRoutes(t *testing.T) {
	t.Run("should handle public route correctly", func(t *testing.T) {
		app := fiber.New()
		initPublicRoutes := func(app *fiber.App) {
			app.Get("/public", func(c *fiber.Ctx) error {
				return c.SendString("Public Route")
			})
		}

		app.Use(middleware.HealthCheck(nil))
		app.Use(middleware.RequestID())
		initPublicRoutes(app)

		req := httptest.NewRequest("GET", "/public", nil)
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode, "Public route should return status 200")
	})

	t.Run("should preserve X-Request-ID if already set by client", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.RequestID())

		app.Get("/request-id", func(c *fiber.Ctx) error {
			return c.SendStatus(http.StatusOK)
		})

		req := httptest.NewRequest("GET", "/request-id", nil)
		req.Header.Set("X-Request-ID", "existing-id")
		resp, err := app.Test(req, -1)
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)
		assert.Equal(t, "existing-id", resp.Header.Get("X-Request-ID"), "Middleware should not overwrite existing X-Request-ID header")
	})

	t.Run("should handle concurrent requests and generate unique IDs", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.RequestID())

		app.Get("/request-id", func(c *fiber.Ctx) error {
			return c.SendStatus(http.StatusOK)
		})

		ids := make(map[string]bool)
		for i := 0; i < 10; i++ {
			req := httptest.NewRequest("GET", "/request-id", nil)
			resp, err := app.Test(req, -1)
			if err != nil {
				t.Fatalf("Request failed: %v", err)
			}
			if resp != nil {
				defer resp.Body.Close()
			}

			assert.Nil(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)

			id := resp.Header.Get("X-Request-ID")
			assert.NotEmpty(t, id, "Each request should have an X-Request-ID header")
			_, exists := ids[id]
			assert.False(t, exists, "Request ID should be unique across concurrent requests")
			ids[id] = true
		}
	})

	t.Run("should set X-Request-ID even on non-matching routes", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.RequestID())

		req := httptest.NewRequest("GET", "/non-existing-route", nil)
		resp, err := app.Test(req, -1)

		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode, "Non-matching route should return 404")
		assert.NotEmpty(t, resp.Header.Get("X-Request-ID"), "X-Request-ID header should still be set on non-matching routes")
	})

	t.Run("should return 500 Internal Server Error if middleware fails", func(t *testing.T) {
		app := fiber.New()

		app.Use(func(c *fiber.Ctx) error {
			return fiber.NewError(http.StatusInternalServerError, "Middleware error")
		})

		app.Get("/request-id", func(c *fiber.Ctx) error {
			return c.SendStatus(http.StatusOK)
		})

		req := httptest.NewRequest("GET", "/request-id", nil)
		resp, err := app.Test(req, -1)

		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode, "Should return 500 if middleware fails")
	})

	t.Run("should generate consistent format for X-Request-ID", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.RequestID())

		app.Get("/request-id", func(c *fiber.Ctx) error {
			return c.SendStatus(http.StatusOK)
		})

		req := httptest.NewRequest("GET", "/request-id", nil)
		resp, err := app.Test(req, -1)

		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		id := resp.Header.Get("X-Request-ID")
		assert.Regexp(t, `^[a-f0-9-]+$`, id, "X-Request-ID should be in a consistent format")
	})

}

func TestInvalidRoutes(t *testing.T) {
	t.Run("should return 404 for an undefined route", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.HealthCheck(nil))

		req := httptest.NewRequest("GET", "/undefined-route", nil)
		resp, err := app.Test(req, -1)

		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode, "Undefined route should return status 404")
	})

	t.Run("should return 405 for valid route with unsupported method", func(t *testing.T) {
		app := fiber.New()
		app.Get("/valid-route", func(c *fiber.Ctx) error {
			return c.SendStatus(http.StatusOK)
		})

		req := httptest.NewRequest("POST", "/valid-route", nil)
		resp, err := app.Test(req, -1)

		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusMethodNotAllowed, resp.StatusCode, "Unsupported method for valid route should return status 405")
	})

	t.Run("should return 404 for routes with query parameters on undefined route", func(t *testing.T) {
		app := fiber.New()
		app.Get("/defined-route", func(c *fiber.Ctx) error {
			return c.SendStatus(http.StatusOK)
		})

		req := httptest.NewRequest("GET", "/undefined-route?query=param", nil)
		resp, err := app.Test(req, -1)

		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode, "Undefined route with query parameters should return status 404")
	})

	t.Run("should return 404 for routes with incorrect parameter format", func(t *testing.T) {
		app := fiber.New()
		app.Get("/route/:id", func(c *fiber.Ctx) error {
			id := c.Params("id")
			if _, err := strconv.Atoi(id); err != nil {
				return c.SendStatus(http.StatusNotFound)
			}
			return c.SendStatus(http.StatusOK)
		})

		req := httptest.NewRequest("GET", "/route/wrong-format-id", nil)
		resp, err := app.Test(req, -1)

		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
		if resp != nil {
			defer resp.Body.Close()
		}

		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode, "Route with incorrect parameter format should return status 404")
	})
	t.Run("should return 404 for routes with similar names or incorrect prefix/suffix", func(t *testing.T) {
		app := fiber.New()
		app.Get("/exact-route", func(c *fiber.Ctx) error {
			return c.SendStatus(http.StatusOK)
		})

		req1 := httptest.NewRequest("GET", "/exact-route-similar", nil)
		req2 := httptest.NewRequest("GET", "/wrong-prefix-route", nil)

		tests := []struct {
			req  *http.Request
			desc string
		}{
			{req1, "similar name"},
			{req2, "incorrect prefix or suffix"},
		}

		for _, test := range tests {
			resp, err := app.Test(test.req, -1)
			if err != nil {
				t.Fatalf("Request failed: %v", err)
			}
			if resp != nil {
				defer resp.Body.Close()
			}

			assert.Nil(t, err)
			assert.Equal(t, http.StatusNotFound, resp.StatusCode, "Route with "+test.desc+" should return status 404")
		}
	})

}
