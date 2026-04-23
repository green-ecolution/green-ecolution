package region

import (
	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
)

func RegisterRoutes(r fiber.Router, svc ports.RegionService) {
	r.Get("/", GetAllRegions(svc))
	r.Get("/:id", GetRegionByID(svc))
}
