package evaluation_test

import (
	"context"
	"errors"
	"net/http"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/evaluation"
	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/service/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestGetEvaluation(t *testing.T) {
	t.Run("should return evaluation data successfully", func(t *testing.T) {
		mockEvaluationService := serviceMock.NewMockEvaluationService(t)
		app := fiber.New()
		handler := evaluation.GetEvaluation(mockEvaluationService)

		mockEvaluationService.EXPECT().GetEvaluation(
			mock.Anything,
		).Return(&entities.Evaluation{}, nil)

		app.Get("/v1/evaluation", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/evaluation", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.EvaluationResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		mockEvaluationService.AssertExpectations(t)
	})

	t.Run("should return 500 when service returns an error", func(t *testing.T) {
		mockEvaluationService := serviceMock.NewMockEvaluationService(t)
		app := fiber.New()
		handler := evaluation.GetEvaluation(mockEvaluationService)

		mockEvaluationService.EXPECT().GetEvaluation(
			mock.Anything,
		).Return(nil, errors.New("service error"))

		app.Get("/v1/evaluation", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/evaluation", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockEvaluationService.AssertExpectations(t)
	})
}
