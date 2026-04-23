package sensor

import (
	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
)

func RegisterRoutes(r fiber.Router, svc ports.SensorService) {
	r.Get("/", GetAllSensors(svc))
	r.Get("/:id", GetSensorByID(svc))
	r.Get("/data/:id", GetAllSensorDataByID(svc))
	r.Delete("/:id", DeleteSensor(svc))
}
