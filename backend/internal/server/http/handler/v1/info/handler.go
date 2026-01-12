package info

import (
	"github.com/gofiber/fiber/v2"
	_ "github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities/mapper/generated"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/errorhandler"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
)

// @Summary		Get application info
// @Description	Retrieves information about the application including version, server configuration, and map settings.
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
func GetAppInfo(svc service.InfoService) fiber.Handler {
	var m mapper.InfoHTTPMapper = &generated.InfoHTTPMapperImpl{}

	return func(c *fiber.Ctx) error {
		domainInfo, err := svc.GetAppInfoResponse(c.Context())
		if err != nil {
			return errorhandler.HandleError(err)
		}

		response := m.ToResponse(domainInfo)
		return c.JSON(response)
	}
}
