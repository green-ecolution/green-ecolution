package evaluation

import (
	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	_ "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/handler/v1/errorhandler"
)

// @Summary		Get evaluation data
// @Description	Retrieves aggregated statistics including tree count, sensor count, cluster count, and watering plan metrics.
// @Id				get-evaluation
// @Tags			Evaluation
// @Produce		json
// @Success		200	{object}	entities.EvaluationResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/evaluation [get]
// @Security		Keycloak
func GetEvaluation(svc ports.EvaluationService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		domainData, err := svc.GetEvaluation(c.Context())
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(mapper.EvaluationFromResponse(domainData))
	}
}
