package info_test

import (
	"context"
	"net/http"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/info"
	serviceMock "github.com/green-ecolution/backend/internal/service/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestRegisterRoutes(t *testing.T) {
	t.Run("/v1/info", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockInfoService := serviceMock.NewMockInfoService(t)
			app := fiber.New()
			info.RegisterRoutes(app, mockInfoService)

			mockInfoService.EXPECT().GetAppInfoResponse(
				mock.Anything,
			).Return(TestInfo, nil)

			// when
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})
	})
}
