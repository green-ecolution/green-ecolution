package treecluster

import (
	"strconv"

	domain "github.com/green-ecolution/backend/internal/entities"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/backend/internal/server/http/entities/mapper/generated"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/errorhandler"
	"github.com/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/backend/internal/utils/pagination"
)

var (
	treeClusterMapper = generated.TreeClusterHTTPMapperImpl{}
)

// @Summary		Get all tree clusters
// @Description	Get all tree clusters
// @Id				get-all-tree-clusters
// @Tags			Tree Cluster
// @Produce		json
// @Success		200	{object}	entities.TreeClusterListResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/cluster [get]
// @Param			page				query	int			false	"Page"
// @Param			limit				query	int			false	"Limit"
// @Param			watering_statuses	query	[]string	false	"Watering statuses"
// @Param			regions				query	[]string	false	"Regions"
// @Param			provider			query	string		false	"Provider"
// @Security		Keycloak
func GetAllTreeClusters(svc service.TreeClusterService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()

		filter, err := fillTreeClusterQueryParams(c)
		if err != nil {
			return errorhandler.HandleError(service.NewError(service.BadRequest, err.Error()))
		}

		domainData, totalCount, err := svc.GetAll(ctx, filter)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := make([]*entities.TreeClusterInListResponse, len(domainData))
		for i, domain := range domainData {
			data[i] = treeClusterMapper.FromInListResponse(domain)
		}

		return c.JSON(entities.TreeClusterListResponse{
			Data:       data,
			Pagination: pagination.Create(ctx, totalCount),
		})
	}
}

// @Summary		Get tree cluster by ID
// @Description	Get tree cluster by ID
// @Id				get-tree-cluster-by-id
// @Tags			Tree Cluster
// @Produce		json
// @Success		200	{object}	entities.TreeClusterResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/cluster/{cluster_id} [get]
// @Param			cluster_id	path	int	true	"Tree Cluster ID"
// @Security		Keycloak
func GetTreeClusterByID(svc service.TreeClusterService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("treecluster_id"))
		if err != nil {
			err := service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		domainData, err := svc.GetByID(ctx, int32(id))

		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(treeClusterMapper.FromResponse(domainData))
	}
}

// @Summary		Create tree cluster
// @Description	Create tree cluster
// @Id				create-tree-cluster
// @Tags			Tree Cluster
// @Produce		json
// @Success		201	{object}	entities.TreeClusterResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/cluster [post]
// @Param			body	body	entities.TreeClusterCreateRequest	true	"Tree Cluster Create Request"
// @Security		Keycloak
func CreateTreeCluster(svc service.TreeClusterService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()

		var req entities.TreeClusterCreateRequest
		if err := c.BodyParser(&req); err != nil {
			return fiber.NewError(fiber.StatusBadRequest, err.Error())
		}

		domainReq := treeClusterMapper.FromCreateRequest(&req)
		domainData, err := svc.Create(ctx, domainReq)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := treeClusterMapper.FromResponse(domainData)
		return c.Status(fiber.StatusCreated).JSON(data)
	}
}

// @Summary		Update tree cluster
// @Description	Update tree cluster
// @Id				update-tree-cluster
// @Tags			Tree Cluster
// @Produce		json
// @Success		200	{object}	entities.TreeClusterResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/cluster/{cluster_id} [put]
// @Param			cluster_id	path	int									true	"Tree Cluster ID"
// @Param			body		body	entities.TreeClusterUpdateRequest	true	"Tree Cluster Update Request"
// @Security		Keycloak
func UpdateTreeCluster(svc service.TreeClusterService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("treecluster_id"))
		if err != nil {
			err := service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		var req entities.TreeClusterUpdateRequest
		if err = c.BodyParser(&req); err != nil {
			return fiber.NewError(fiber.StatusBadRequest, err.Error())
		}

		domainReq := treeClusterMapper.FromUpdateRequest(&req)
		domainData, err := svc.Update(ctx, int32(id), domainReq)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(treeClusterMapper.FromResponse(domainData))
	}
}

// @Summary		Delete tree cluster
// @Description	Delete tree cluster
// @Id				delete-tree-cluster
// @Tags			Tree Cluster
// @Produce		json
// @Success		204
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/cluster/{cluster_id} [delete]
// @Param			cluster_id	path	int	true	"Tree Cluster ID"
// @Security		Keycloak
func DeleteTreeCluster(svc service.TreeClusterService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("treecluster_id"))
		if err != nil {
			err := service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		err = svc.Delete(ctx, int32(id))
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.SendStatus(fiber.StatusNoContent)
	}
}

func fillTreeClusterQueryParams(c *fiber.Ctx) (domain.TreeClusterQuery, error) {
	var filter domain.TreeClusterQuery

	if err := c.QueryParser(&filter); err != nil {
		return domain.TreeClusterQuery{}, err
	}

	return filter, nil
}
