package evaluation

import (
	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
)

func RegisterRoutes(r fiber.Router, svc ports.EvaluationService) {
	r.Get("/", GetEvaluation(svc))
}
