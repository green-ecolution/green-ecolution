package info

import (
	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/handler/v1/errorhandler"
)

// @Summary		Get application info
// @Description	Retrieves basic application information including version and git info.
// @Id				get-app-info
// @Tags			Info
// @Produce		json
// @Success		200	{object}	entities.AppInfoResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/info [get]
// @Security		Keycloak
func GetAppInfo(svc ports.InfoService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		domainInfo, err := svc.GetAppInfoResponse(c.Context())
		if err != nil {
			return errorhandler.HandleError(err)
		}

		response := mapper.InfoToResponse(domainInfo)
		return c.JSON(response)
	}
}

// @Summary		Get map configuration
// @Description	Retrieves map center and bounding box configuration.
// @Id				get-map-info
// @Tags			Info
// @Produce		json
// @Success		200	{object}	entities.MapResponse
// @Failure		500	{object}	HTTPError
// @Router			/v1/info/map [get]
func GetMapInfo(svc ports.InfoService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		mapInfo, err := svc.GetMapInfo(c.Context())
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(entities.MapResponse{
			Center: mapInfo.Center,
			BBox:   mapInfo.BBox,
		})
	}
}

// @Summary		Get server information
// @Description	Retrieves server details including hostname, IP, and uptime. Requires authentication.
// @Id				get-server-info
// @Tags			Info
// @Produce		json
// @Success		200	{object}	entities.ServerResponse
// @Failure		401	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/info/server [get]
// @Security		Keycloak
func GetServerInfo(svc ports.InfoService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		serverInfo, err := svc.GetServerInfo(c.Context())
		if err != nil {
			return errorhandler.HandleError(err)
		}

		response := mapper.InfoServerToResponse(serverInfo)
		return c.JSON(response)
	}
}

// @Summary		Get services status
// @Description	Retrieves health status of all backend services with response times.
// @Id				get-services-status
// @Tags			Info
// @Produce		json
// @Success		200	{object}	entities.ServicesResponse
// @Failure		500	{object}	HTTPError
// @Router			/v1/info/services [get]
func GetServicesStatus(svc ports.InfoService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		services, err := svc.GetServices(c.Context())
		if err != nil {
			return errorhandler.HandleError(err)
		}

		response := mapper.InfoServicesToResponse(services)
		return c.JSON(response)
	}
}

// @Summary		Get data statistics
// @Description	Retrieves counts of various entities in the system.
// @Id				get-data-statistics
// @Tags			Info
// @Produce		json
// @Success		200	{object}	entities.DataStatisticsResponse
// @Failure		500	{object}	HTTPError
// @Router			/v1/info/statistics [get]
func GetStatistics(svc ports.InfoService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		stats, err := svc.GetStatistics(c.Context())
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(entities.DataStatisticsResponse{
			TreeCount:         stats.TreeCount,
			SensorCount:       stats.SensorCount,
			VehicleCount:      stats.VehicleCount,
			TreeClusterCount:  stats.TreeClusterCount,
			WateringPlanCount: stats.WateringPlanCount,
		})
	}
}
