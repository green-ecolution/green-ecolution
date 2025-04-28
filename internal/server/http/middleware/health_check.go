package middleware

import (
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/healthcheck"
	"github.com/green-ecolution/backend/internal/service"
)

func HealthCheck(svc service.ServicesInterface) fiber.Handler {
	return healthcheck.New(healthcheck.Config{
		LivenessProbe: func(_ *fiber.Ctx) bool {
			return true
		},
		LivenessEndpoint: "/health",
		ReadinessProbe: func(_ *fiber.Ctx) bool {
			return svc.AllServicesReady()
		},
		ReadinessEndpoint: "/ready",
	})
}
