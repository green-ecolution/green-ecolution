package tree

import (
	"strconv"
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
	treeMapper   = generated.TreeHTTPMapperImpl{}
	sensorMapper = generated.SensorHTTPMapperImpl{}
)

// @Summary		Get all trees
// @Description	Get all trees
// @Id				get-all-trees
// @Tags			Tree
// @Produce		json
// @Success		200	{object}	entities.TreeListResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree [get]
// @Param			page				query	int			false	"Page"
// @Param			limit				query	int			false	"Limit"
// @Param			provider			query	string		false	"Provider"
// @Param			watering_statuses	query	[]string	false	"watering status (good, moderate, bad)"
// @Param			planting_years		query	[]int		false	"planting_years"
// @Param			has_cluster			query	bool		false	"has cluster"
// @Security		Keycloak
func GetAllTrees(svc service.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()

		filter, err := fillTreeQueryParams(c)
		if err != nil {
			return errorhandler.HandleError(service.NewError(service.BadRequest, err.Error()))
		}

		domainData, totalCount, err := svc.GetAll(ctx, filter)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := make([]*entities.TreeResponse, len(domainData))
		for i, domain := range domainData {
			data[i] = mapTreeToDto(domain)
		}

		return c.JSON(entities.TreeListResponse{
			Data:       data,
			Pagination: pagination.Create(ctx, totalCount),
		})
	}
}

// @Summary		Get tree by ID
// @Description	Get tree by ID
// @Id				get-trees
// @Tags			Tree
// @Produce		json
// @Success		200	{object}	entities.TreeResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree/{tree_id} [get]
// @Param			tree_id	path	int	false	"Tree ID"
// @Security		Keycloak
func GetTreeByID(svc service.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		idStr := c.Params("id")

		id, err := strconv.Atoi(idStr)
		if err != nil {
			err = service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		domainData, err := svc.GetByID(ctx, int32(id))
		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := mapTreeToDto(domainData)

		return c.JSON(data)
	}
}

// @Summary		Get tree by sensor ID
// @Description	Get tree by sensor ID
// @Id				get-tree-by-sensor-id
// @Tags			Tree Sensor
// @Produce		json
// @Success		200	{object}	entities.TreeResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree/sensor/{sensor_id} [get]
// @Param			sensor_id	path	string	false	"Sensor ID"
// @Security		Keycloak
func GetTreeBySensorID(svc service.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id := strings.Clone(c.Params("sensor_id"))

		domainData, err := svc.GetBySensorID(ctx, id)
		if err != nil {
			return errorhandler.HandleError(err)
		}
		data := mapTreeToDto(domainData)

		return c.JSON(data)
	}
}

// @Summary		Create tree
// @Description	Create tree
// @Id				create-tree
// @Tags			Tree
// @Produce		json
// @Success		200	{object}	entities.TreeResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree [post]
// @Security		Keycloak
// @Param			body	body	entities.TreeCreateRequest	true	"Tree to create"
func CreateTree(svc service.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		var req entities.TreeCreateRequest
		if err := c.BodyParser(&req); err != nil {
			return fiber.NewError(fiber.StatusBadRequest, err.Error())
		}

		domainReq := treeMapper.FromCreateRequest(&req)
		domainData, err := svc.Create(ctx, domainReq)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := mapTreeToDto(domainData)
		return c.Status(fiber.StatusCreated).JSON(data)
	}
}

// @Summary		Update tree
// @Description	Update tree
// @Id				update-tree
// @Tags			Tree
// @Produce		json
// @Success		200	{object}	entities.TreeResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree/{tree_id} [put]
// @Security		Keycloak
// @Param			tree_id	path	int							false	"Tree ID"
// @Param			body	body	entities.TreeUpdateRequest	true	"Tree to update"
func UpdateTree(svc service.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("id"))
		if err != nil {
			err = service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}
		var req entities.TreeUpdateRequest
		if err = c.BodyParser(&req); err != nil {
			return fiber.NewError(fiber.StatusBadRequest, err.Error())
		}
		domainReq := treeMapper.FromUpdateRequest(&req)
		domainData, err := svc.Update(ctx, int32(id), domainReq)
		if err != nil {
			return errorhandler.HandleError(err)
		}
		data := mapTreeToDto(domainData)
		return c.JSON(data)
	}
}

// @Summary		Delete tree
// @Description	Delete tree
// @Id				delete-tree
// @Tags			Tree
// @Produce		json
// @Success		204
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree/{tree_id} [delete]
// @Param			tree_id	path	int	false	"Tree ID"
// @Security		Keycloak
func DeleteTree(svc service.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("id"))
		if err != nil {
			err = service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}
		err = svc.Delete(ctx, int32(id))
		if err != nil {
			return errorhandler.HandleError(err)
		}
		return c.SendStatus(fiber.StatusNoContent)
	}
}

func mapTreeToDto(t *domain.Tree) *entities.TreeResponse {
	dto := treeMapper.FromResponse(t)
	dto.Sensor = sensorMapper.FromResponse(t.Sensor)

	return dto
}

func fillTreeQueryParams(c *fiber.Ctx) (domain.TreeQuery, error) {
	var filter domain.TreeQuery

	if err := c.QueryParser(&filter); err != nil {
		return domain.TreeQuery{}, err
	}

	return filter, nil
}
