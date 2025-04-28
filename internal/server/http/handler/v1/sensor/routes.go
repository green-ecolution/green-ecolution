package sensor

import (
	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/backend/internal/service"
)

func RegisterRoutes(r fiber.Router, svc service.SensorService) {
	r.Get("/", GetAllSensors(svc))
	r.Get("/:id", GetSensorByID(svc))
	r.Get("/data/:id", GetAllSensorDataByID(svc))
	r.Delete("/:id", DeleteSensor(svc))
}
