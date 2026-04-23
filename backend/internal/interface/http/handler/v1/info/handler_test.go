package info_test

import (
	"context"
	"errors"
	"net/http"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/application/_mock"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/handler/v1/info"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func TestGetAppInfo(t *testing.T) {
	t.Run("should return app info successfully", func(t *testing.T) {
		mockInfoService := serviceMock.NewMockInfoService(t)
		app := fiber.New()
		handler := info.GetAppInfo(mockInfoService)

		mockInfoService.EXPECT().GetAppInfoResponse(
			mock.Anything,
		).Return(TestInfo, nil)

		app.Get("/v1/info", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/info", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.AppInfoResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		mockInfoService.AssertExpectations(t)
	})

	t.Run("should return 500 when service returns an error", func(t *testing.T) {
		mockInfoService := serviceMock.NewMockInfoService(t)
		app := fiber.New()
		handler := info.GetAppInfo(mockInfoService)

		mockInfoService.EXPECT().GetAppInfoResponse(
			mock.Anything,
		).Return(nil, errors.New("service error"))

		app.Get("/v1/info", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/info", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockInfoService.AssertExpectations(t)
	})
}
