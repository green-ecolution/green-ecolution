package wateringplan

import (
	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
)

func RegisterRoutes(r fiber.Router, svc service.WateringPlanService) {
	r.Get("/", GetAllWateringPlans(svc))
	r.Get("/:id", GetWateringPlanByID(svc))
	r.Post("/", CreateWateringPlan(svc))
	r.Put("/:id", UpdateWateringPlan(svc))
	r.Delete("/:id", DeleteWateringPlan(svc))
	r.Post("/route/preview", CreatePreviewRoute(svc))
	r.Get("/route/gpx/:gpx_name", GetGpxFile(svc))
}
