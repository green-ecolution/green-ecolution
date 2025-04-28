package region

import (
	"strconv"

	"github.com/gofiber/fiber/v2"
	domain "github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/errorhandler"
	"github.com/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/backend/internal/utils"
	"github.com/green-ecolution/backend/internal/utils/pagination"
)

// @Summary		Get all regions
// @Description	Get all regions
// @Tags			Region
// @Produce		json
// @Success		200		{object}	entities.RegionListResponse
// @Failure		400		{object}	HTTPError
// @Failure		500		{object}	HTTPError
// @Param			page	query		int	false	"Page"
// @Param			limit	query		int	false	"Limit"
// @Router			/v1/region [get]
// @Security		Keycloak
func GetAllRegions(svc service.RegionService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		r, totalCount, err := svc.GetAll(ctx)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		dto := utils.Map(r, func(region *domain.Region) *entities.RegionResponse {
			return &entities.RegionResponse{
				ID:   region.ID,
				Name: region.Name,
			}
		})

		return c.JSON(entities.RegionListResponse{
			Data:       dto,
			Pagination: pagination.Create(ctx, totalCount),
		})
	}
}

// @Summary		Get a region by ID
// @Description	Get a region by ID
// @Tags			Region
// @Produce		json
// @Success		200	{object}	entities.RegionResponse
// @Failure		400	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Param			id	path		string	true	"Region ID"
// @Router			/v1/region/{id} [get]
// @Security		Keycloak
func GetRegionByID(svc service.RegionService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		id, err := strconv.Atoi(c.Params("id"))
		if err != nil {
			err := service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		// linter complains about overflows, but we are sure that the ID is not going to be bigger than int32
		//nolint: gosec
		r, err := svc.GetByID(c.Context(), int32(id))
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(r)
	}
}
