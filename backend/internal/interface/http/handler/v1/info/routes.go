package info

import (
	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
)

func RegisterRoutes(r fiber.Router, svc ports.InfoService) {
	r.Get("/", GetAppInfo(svc))
	r.Get("/map", GetMapInfo(svc))
	r.Get("/server", GetServerInfo(svc))
	r.Get("/services", GetServicesStatus(svc))
	r.Get("/statistics", GetStatistics(svc))
}
