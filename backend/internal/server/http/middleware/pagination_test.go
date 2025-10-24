package middleware

import (
	"encoding/json"
	"net/http/httptest"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/stretchr/testify/assert"
)

func TestPaginationMiddleware(t *testing.T) {
	app := fiber.New()

	app.Get("/pagination-test", PaginationMiddleware(), func(c *fiber.Ctx) error {
		page := c.Locals("page")
		limit := c.Locals("limit")
		return c.JSON(fiber.Map{
			"page":  page,
			"limit": limit,
		})
	})

	t.Run("should set default values when no page and limit are provided", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/pagination-test", nil)
		resp, err := app.Test(req)
		assert.NoError(t, err)
		defer resp.Body.Close()

		var result map[string]interface{}
		assert.NoError(t, json.NewDecoder(resp.Body).Decode(&result))

		assert.Equal(t, float64(1), result["page"])
		assert.Equal(t, float64(-1), result["limit"])
	})

	t.Run("should use provided page and limit query parameters", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/pagination-test?page=2&limit=10", nil)
		resp, err := app.Test(req)
		assert.NoError(t, err)
		defer resp.Body.Close()

		var result map[string]interface{}
		assert.NoError(t, json.NewDecoder(resp.Body).Decode(&result))

		assert.Equal(t, float64(2), result["page"])
		assert.Equal(t, float64(10), result["limit"])
	})

	t.Run("should return bad request error for invalid page format", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/pagination-test?page=abc&limit=10", nil)
		resp, err := app.Test(req)
		assert.NoError(t, err)
		defer resp.Body.Close()

		assert.Equal(t, 400, resp.StatusCode)
	})

	t.Run("should return bad request error for zero page format", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/pagination-test?page=0&limit=10", nil)
		resp, err := app.Test(req)
		assert.NoError(t, err)
		defer resp.Body.Close()

		assert.Equal(t, 400, resp.StatusCode)
	})

	t.Run("should return bad request error for invalid limit format", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/pagination-test?page=2&limit=xyz", nil)
		resp, err := app.Test(req)
		assert.NoError(t, err)
		defer resp.Body.Close()

		assert.Equal(t, 400, resp.StatusCode)
	})

	t.Run("should return bad request error for negative limit", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/pagination-test?page=2&limit=-10", nil)
		resp, err := app.Test(req)
		assert.NoError(t, err)
		defer resp.Body.Close()

		assert.Equal(t, 400, resp.StatusCode)
	})

	t.Run("should return bad request error for zero limit", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/pagination-test?page=2&limit=0", nil)
		resp, err := app.Test(req)
		assert.NoError(t, err)
		defer resp.Body.Close()

		assert.Equal(t, 400, resp.StatusCode)
	})
}
