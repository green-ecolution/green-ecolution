package sensor

import (
	"strings"

	"github.com/gofiber/fiber/v2"
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities/mapper/generated"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/errorhandler"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/pagination"
)

var (
	sensorMapper = generated.SensorHTTPMapperImpl{}
)

// @Summary		Get all sensors
// @Description	Retrieves a paginated list of all sensors. Supports filtering by provider.
// @Id				get-all-sensors
// @Tags			Sensor
// @Produce		json
// @Success		200	{object}	entities.SensorListResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/sensor [get]
// @Param			page		query	int		false	"Page number for pagination"
// @Param			limit		query	int		false	"Number of items per page"
// @Param			provider	query	string	false	"Filter by data provider"
// @Security		Keycloak
func GetAllSensors(svc service.SensorService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		var query domain.Query

		if err := c.QueryParser(&query); err != nil {
			return errorhandler.HandleError(err)
		}

		domainData, totalCount, err := svc.GetAll(ctx, query)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := make([]*entities.SensorResponse, len(domainData))
		for i, domain := range domainData {
			data[i] = mapToDto(domain)
		}

		return c.JSON(entities.SensorListResponse{
			Data:       data,
			Pagination: pagination.Create(ctx, totalCount),
		})
	}
}

// @Summary		Get sensor by ID
// @Description	Retrieves detailed information about a specific sensor including its latest data readings.
// @Id				get-sensor-by-id
// @Tags			Sensor
// @Produce		json
// @Success		200	{object}	entities.SensorResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/sensor/{sensor_id} [get]
// @Param			sensor_id	path	string	true	"Sensor ID"
// @Security		Keycloak
func GetSensorByID(svc service.SensorService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id := strings.Clone(c.Params("id"))
		if id == "" {
			err := service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		domainData, err := svc.GetByID(ctx, id)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(mapToDto(domainData))
	}
}

// @Summary		Get all sensor data by ID
// @Description	Retrieves the complete history of data readings for a specific sensor.
// @Id				get-all-sensor-data-by-id
// @Tags			Sensor
// @Produce		json
// @Success		200	{object}	entities.SensorDataListResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/sensor/data/{sensor_id} [get]
// @Param			sensor_id	path	string	true	"Sensor ID"
// @Security		Keycloak
func GetAllSensorDataByID(svc service.SensorService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id := strings.Clone(c.Params("id"))
		if id == "" {
			err := service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		domainData, err := svc.GetAllDataByID(ctx, id)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := make([]*entities.SensorDataResponse, len(domainData))
		for i, domain := range domainData {
			data[i] = sensorMapper.FromDataResponse(domain)
		}

		return c.JSON(entities.SensorDataListResponse{
			Data: data,
		})
	}
}

// @Summary		Delete sensor
// @Description	Permanently deletes a sensor and all its associated data readings.
// @Id				delete-sensor
// @Tags			Sensor
// @Produce		json
// @Success		204
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/sensor/{sensor_id} [delete]
// @Param			sensor_id	path	string	true	"Sensor ID"
// @Security		Keycloak
func DeleteSensor(svc service.SensorService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id := strings.Clone(c.Params("id"))
		if id == "" {
			err := service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		err := svc.Delete(ctx, id)

		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.SendStatus(fiber.StatusNoContent)
	}
}

// TODO: Create / Update Sensor

func mapToDto(t *domain.Sensor) *entities.SensorResponse {
	dto := sensorMapper.FromResponse(t)
	return dto
}
