package evaluation_test

import (
	"context"
	"net/http"
	"testing"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/evaluation"
	serviceMock "github.com/green-ecolution/backend/internal/service/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestRegisterRoutes(t *testing.T) {
	t.Run("/v1/evaluation", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockEvaluationService := serviceMock.NewMockEvaluationService(t)
			app := fiber.New()
			evaluation.RegisterRoutes(app, mockEvaluationService)

			mockEvaluationService.EXPECT().GetEvaluation(
				mock.Anything,
			).Return(&entities.Evaluation{}, nil)

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
