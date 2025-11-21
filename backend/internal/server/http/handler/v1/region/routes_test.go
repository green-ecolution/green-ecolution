package region

import (
	"context"
	"net/http"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/middleware"
	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/service/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestRegisterRoutes(t *testing.T) {
	t.Run("/v1/region", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockRegionService := serviceMock.NewMockRegionService(t)
			app := fiber.New()
			app.Use(middleware.PaginationMiddleware())
			RegisterRoutes(app, mockRegionService)

			ctx := context.WithValue(context.Background(), "page", int32(1))
			ctx = context.WithValue(ctx, "limit", int32(-1))

			expectedRegions := []*entities.Region{
				{ID: 1, Name: "Region A"},
				{ID: 2, Name: "Region B"},
			}

			mockRegionService.EXPECT().GetAll(
				mock.Anything,
			).Return(expectedRegions, int64(len(expectedRegions)), nil)

			// when
			req, _ := http.NewRequestWithContext(ctx, http.MethodGet, "/", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})
	})

	t.Run("/v1/region/:id", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockRegionService := serviceMock.NewMockRegionService(t)
			app := fiber.New()
			RegisterRoutes(app, mockRegionService)

			mockRegionService.EXPECT().GetByID(
				mock.Anything,
				int32(1),
			).Return(&entities.Region{ID: 1, Name: "Region A"}, nil)

			// when
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/1", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})
	})
}
