package tree

import (
	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
)

func RegisterRoutes(r fiber.Router, svc ports.TreeService) {
	r.Get("/", GetAllTrees(svc))
	r.Get("/planting-years", GetPlantingYears(svc))
	r.Get("/nearest", GetNearestTrees(svc))
	r.Get("/:id", GetTreeByID(svc))
	r.Put("/:id", UpdateTree(svc))
	r.Post("/", CreateTree(svc))
	r.Delete("/:id", DeleteTree(svc))

	r.Get("/sensor/:sensor_id", GetTreeBySensorID(svc))
}
