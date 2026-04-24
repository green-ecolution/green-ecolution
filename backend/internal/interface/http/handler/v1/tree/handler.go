package tree

import (
	"strconv"
	"strings"

	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities/mapper"
	handler "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/handler/v1"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/handler/v1/errorhandler"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/pagination"
)

// @Summary		Get all trees
// @Description	Retrieves a paginated list of all trees. Supports filtering by provider, watering status, planting year, and cluster association.
// @Id				get-all-trees
// @Tags			Tree
// @Produce		json
// @Success		200	{object}	entities.TreeListResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree [get]
// @Param			page				query	int			false	"Page number for pagination"
// @Param			limit				query	int			false	"Number of items per page"
// @Param			provider			query	string		false	"Filter by data provider"
// @Param			watering_statuses	query	[]string	false	"Filter by watering status (good, moderate, bad)"
// @Param			planting_years		query	[]int		false	"Filter by planting years"
// @Param			has_cluster			query	bool		false	"Filter trees that belong to a cluster"
// @Security		Keycloak
func GetAllTrees(svc ports.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()

		filter, err := fillTreeQueryParams(c)
		if err != nil {
			return errorhandler.HandleError(ports.NewError(ports.BadRequest, err.Error()))
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
// @Description	Retrieves detailed information about a specific tree including its sensor data and cluster association.
// @Id				get-tree-by-id
// @Tags			Tree
// @Produce		json
// @Success		200	{object}	entities.TreeResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree/{tree_id} [get]
// @Param			tree_id	path	int	true	"Tree ID"
// @Security		Keycloak
func GetTreeByID(svc ports.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		idStr := c.Params("id")

		id, err := strconv.Atoi(idStr)
		if err != nil {
			err = ports.NewError(ports.BadRequest, "invalid ID format")
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
// @Description	Retrieves the tree associated with a specific sensor. Useful for looking up tree information from sensor data.
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
// @Param			sensor_id	path	string	true	"Sensor ID"
// @Security		Keycloak
func GetTreeBySensorID(svc ports.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		rawID := strings.Clone(c.Params("sensor_id"))

		sensorID, err := sensor.NewSensorID(rawID)
		if err != nil {
			return errorhandler.HandleError(ports.NewError(ports.BadRequest, err.Error()))
		}

		domainData, err := svc.GetBySensorID(ctx, sensorID)
		if err != nil {
			return errorhandler.HandleError(err)
		}
		data := mapTreeToDto(domainData)

		return c.JSON(data)
	}
}

// @Summary		Create tree
// @Description	Creates a new tree with the provided data. Optionally associates a sensor and cluster.
// @Id				create-tree
// @Tags			Tree
// @Accept			json
// @Produce		json
// @Success		201	{object}	entities.TreeResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree [post]
// @Security		Keycloak
// @Param			body	body	entities.TreeCreateRequest	true	"Tree data to create"
func CreateTree(svc ports.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		req, err := handler.BindAndValidate[entities.TreeCreateRequest](c)
		if err != nil {
			return err
		}

		domainReq, err := mapper.TreeFromCreateRequest(req)
		if err != nil {
			return errorhandler.HandleError(ports.NewError(ports.BadRequest, err.Error()))
		}
		domainData, err := svc.Create(ctx, domainReq)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := mapTreeToDto(domainData)
		return c.Status(fiber.StatusCreated).JSON(data)
	}
}

// @Summary		Update tree
// @Description	Updates an existing tree with the provided data. All fields in the request body will overwrite existing values.
// @Id				update-tree
// @Tags			Tree
// @Accept			json
// @Produce		json
// @Success		200	{object}	entities.TreeResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree/{tree_id} [put]
// @Security		Keycloak
// @Param			tree_id	path	int							true	"Tree ID"
// @Param			body	body	entities.TreeUpdateRequest	true	"Tree data to update"
func UpdateTree(svc ports.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("id"))
		if err != nil {
			err = ports.NewError(ports.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}
		req, err := handler.BindAndValidate[entities.TreeUpdateRequest](c)
		if err != nil {
			return err
		}
		domainReq, err := mapper.TreeFromUpdateRequest(req)
		if err != nil {
			return errorhandler.HandleError(ports.NewError(ports.BadRequest, err.Error()))
		}
		domainData, err := svc.Update(ctx, int32(id), domainReq)
		if err != nil {
			return errorhandler.HandleError(err)
		}
		data := mapTreeToDto(domainData)
		return c.JSON(data)
	}
}

// @Summary		Delete tree
// @Description	Permanently deletes a tree and removes its sensor association if present.
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
// @Param			tree_id	path	int	true	"Tree ID"
// @Security		Keycloak
func DeleteTree(svc ports.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("id"))
		if err != nil {
			err = ports.NewError(ports.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}
		err = svc.Delete(ctx, int32(id))
		if err != nil {
			return errorhandler.HandleError(err)
		}
		return c.SendStatus(fiber.StatusNoContent)
	}
}

// @Summary		Get distinct planting years
// @Description	Retrieves a list of all distinct planting years from trees in the database.
// @Id				get-planting-years
// @Tags			Tree
// @Produce		json
// @Success		200	{array}		int
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree/planting-years [get]
// @Security		Keycloak
func GetPlantingYears(svc ports.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		years, err := svc.GetPlantingYears(ctx)
		if err != nil {
			return errorhandler.HandleError(err)
		}
		return c.JSON(years)
	}
}

type nearestTreesQuery struct {
	Lat   *float64 `query:"lat"`
	Lng   *float64 `query:"lng"`
	Limit int32    `query:"limit"`
}

// @Summary		Get nearest trees
// @Description	Finds the nearest trees to a given GPS coordinate, sorted by distance (in meters).
// @Description	Trees with an assigned sensor include their sensor data but are NOT excluded from results.
// @Description	The search radius is server-controlled via configuration.
// @Id				get-nearest-trees
// @Tags			Tree
// @Produce		json
// @Success		200	{object}	entities.NearestTreesResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/tree/nearest [get]
// @Param			lat		query	number	true	"Latitude (-90 to 90)"
// @Param			lng		query	number	true	"Longitude (-180 to 180)"
// @Param			limit	query	int		false	"Max number of results (server-clamped)"
// @Security		Keycloak
func GetNearestTrees(svc ports.TreeService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()

		var q nearestTreesQuery
		if err := c.QueryParser(&q); err != nil {
			return errorhandler.HandleError(ports.NewError(ports.BadRequest, "invalid query parameters"))
		}
		if q.Lat == nil || q.Lng == nil {
			return errorhandler.HandleError(ports.NewError(ports.BadRequest, "lat and lng query parameters are required"))
		}
		coord, err := shared.NewCoordinate(*q.Lat, *q.Lng)
		if err != nil {
			return errorhandler.HandleError(ports.NewError(ports.BadRequest, err.Error()))
		}

		results, err := svc.GetNearestTrees(ctx, coord, q.Limit)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := make([]*entities.TreeWithDistanceResponse, len(results))
		for i, r := range results {
			data[i] = &entities.TreeWithDistanceResponse{
				Tree:           mapTreeToDto(r.Tree),
				DistanceMeters: r.Distance.Meters(),
			}
		}
		return c.JSON(entities.NearestTreesResponse{Data: data})
	}
}

func mapTreeToDto(t *tree.Tree) *entities.TreeResponse {
	return mapper.TreeFromResponse(t)
}

func fillTreeQueryParams(c *fiber.Ctx) (tree.TreeQuery, error) {
	var filter tree.TreeQuery

	if err := c.QueryParser(&filter); err != nil {
		return tree.TreeQuery{}, err
	}

	return filter, nil
}
