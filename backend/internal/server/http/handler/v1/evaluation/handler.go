package evaluation

import (
	"github.com/gofiber/fiber/v2"
	_ "github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities/mapper/generated"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/errorhandler"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
)

var (
	evaluationMapper = generated.EvaluationHTTPMapperImpl{}
)

// @Summary		Get evaluation data
// @Description	Get evaluation values such as tree count, sensor count, etc.
// @Id				get-evaluation
// @Tags			Evaluation
// @Produce		json
// @Success		200	{object}	entities.EvaluationResponse
// @Failure		400	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/evaluation [get]
// @Security		Keycloak
func GetEvaluation(svc service.EvaluationService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		domainData, err := svc.GetEvaluation(c.Context())
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(evaluationMapper.FromResponse(domainData))
	}
}
