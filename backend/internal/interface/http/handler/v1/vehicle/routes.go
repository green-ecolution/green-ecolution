package vehicle

import (
	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
)

func RegisterRoutes(r fiber.Router, svc ports.VehicleService) {
	r.Get("/", GetAllVehicles(svc))
	r.Get("/archive", GetArchiveVehicles(svc))
	r.Get("/plate/:plate", GetVehicleByPlate(svc))
	r.Get("/:id", GetVehicleByID(svc))
	r.Post("/", CreateVehicle(svc))
	r.Post("/archive/:id", ArchiveVehicle(svc))
	r.Put("/:id", UpdateVehicle(svc))
	r.Delete("/:id", DeleteVehicle(svc))
}
