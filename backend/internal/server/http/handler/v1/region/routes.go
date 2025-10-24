package region

import (
	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/backend/internal/service"
)

func RegisterRoutes(r fiber.Router, svc service.RegionService) {
	r.Get("/", GetAllRegions(svc))
	r.Get("/:id", GetRegionByID(svc))
}
