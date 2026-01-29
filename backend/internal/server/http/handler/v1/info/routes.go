package info

import (
	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
)

func RegisterRoutes(r fiber.Router, svc service.InfoService) {
	r.Get("/", GetAppInfo(svc))
	r.Get("/map", GetMapInfo(svc))
	r.Get("/server", GetServerInfo(svc))
	r.Get("/services", GetServicesStatus(svc))
	r.Get("/statistics", GetStatistics(svc))
}
