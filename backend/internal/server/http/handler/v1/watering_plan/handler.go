package wateringplan

import (
	"fmt"
	"io"
	"strconv"
	"strings"

	"github.com/gofiber/fiber/v2"
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities/mapper/generated"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/errorhandler"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/pagination"
)

var (
	wateringPlanMapper = generated.WateringPlanHTTPMapperImpl{}
)

// @Summary		Get all watering plans
// @Description	Retrieves a paginated list of all watering plans. Supports filtering by provider.
// @Id				get-all-watering-plans
// @Tags			Watering Plan
// @Produce		json
// @Success		200	{object}	entities.WateringPlanListResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/watering-plan [get]
// @Param			page		query	int		false	"Page number for pagination"
// @Param			limit		query	int		false	"Number of items per page"
// @Param			provider	query	string	false	"Filter by data provider"
// @Security		Keycloak
func GetAllWateringPlans(svc service.WateringPlanService) fiber.Handler {
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

		data := make([]*entities.WateringPlanInListResponse, len(domainData))
		for i, domain := range domainData {
			data[i] = wateringPlanMapper.FromInListResponse(domain)
		}

		return c.JSON(entities.WateringPlanListResponse{
			Data:       data,
			Pagination: pagination.Create(ctx, totalCount),
		})
	}
}

// @Summary		Get watering plan by ID
// @Description	Retrieves detailed information about a specific watering plan including assigned clusters, vehicles, and route.
// @Id				get-watering-plan-by-id
// @Tags			Watering Plan
// @Produce		json
// @Success		200	{object}	entities.WateringPlanResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/watering-plan/{id} [get]
// @Param			id	path	int	true	"Watering Plan ID"
// @Security		Keycloak
func GetWateringPlanByID(svc service.WateringPlanService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("id"))
		if err != nil {
			err := service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		domainData, err := svc.GetByID(ctx, int32(id))

		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(wateringPlanMapper.FromResponse(domainData))
	}
}

// @Summary		Create watering plan
// @Description	Creates a new watering plan with specified tree clusters, vehicles, and generates an optimized route.
// @Id				create-watering-plan
// @Tags			Watering Plan
// @Accept			json
// @Produce		json
// @Success		201	{object}	entities.WateringPlanResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/watering-plan [post]
// @Param			body	body	entities.WateringPlanCreateRequest	true	"Watering plan data to create"
// @Security		Keycloak
func CreateWateringPlan(svc service.WateringPlanService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()

		var req entities.WateringPlanCreateRequest
		if err := c.BodyParser(&req); err != nil {
			return fiber.NewError(fiber.StatusBadRequest, err.Error())
		}

		domainReq := wateringPlanMapper.FromCreateRequest(&req)
		domainData, err := svc.Create(ctx, domainReq)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := wateringPlanMapper.FromResponse(domainData)
		return c.Status(fiber.StatusCreated).JSON(data)
	}
}

// @Summary		Update watering plan
// @Description	Updates an existing watering plan. Can modify clusters, vehicles, and regenerates the route if needed.
// @Id				update-watering-plan
// @Tags			Watering Plan
// @Accept			json
// @Produce		json
// @Success		200	{object}	entities.WateringPlanResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/watering-plan/{id} [put]
// @Param			id		path	int									true	"Watering Plan ID"
// @Param			body	body	entities.WateringPlanUpdateRequest	true	"Watering plan data to update"
// @Security		Keycloak
func UpdateWateringPlan(svc service.WateringPlanService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("id"))
		if err != nil {
			err := service.NewError(service.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		var req entities.WateringPlanUpdateRequest
		if err = c.BodyParser(&req); err != nil {
			return fiber.NewError(fiber.StatusBadRequest, err.Error())
		}

		domainReq := wateringPlanMapper.FromUpdateRequest(&req)
		domainData, err := svc.Update(ctx, int32(id), domainReq)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(wateringPlanMapper.FromResponse(domainData))
	}
}

// @Summary		Delete watering plan
// @Description	Permanently deletes a watering plan and its associated route data.
// @Id				delete-watering-plan
// @Tags			Watering Plan
// @Produce		json
// @Success		204
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/watering-plan/{id} [delete]
// @Param			id	path	int	true	"Watering Plan ID"
// @Security		Keycloak
func DeleteWateringPlan(svc service.WateringPlanService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("id"))
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

// @Summary		Generate preview route
// @Description	Generates a preview of the optimized route for the given vehicles and tree clusters without creating a watering plan.
// @Id				create-preview-route
// @Tags			Watering Plan
// @Accept			json
// @Produce		json
// @Success		200		{object}	entities.GeoJSON
// @Failure		400		{object}	HTTPError
// @Failure		401		{object}	HTTPError
// @Failure		403		{object}	HTTPError
// @Failure		500		{object}	HTTPError
// @Param			body	body		entities.RouteRequest	true	"Route preview request with vehicles and clusters"
// @Router			/v1/watering-plan/route/preview [post]
// @Security		Keycloak
func CreatePreviewRoute(svc service.WateringPlanService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		var req entities.RouteRequest
		if err := c.BodyParser(&req); err != nil {
			return fiber.NewError(fiber.StatusBadRequest, err.Error())
		}

		domainGeo, err := svc.PreviewRoute(ctx, req.TransporterID, req.TrailerID, req.TreeClusterIDs)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(entities.GeoJSON{
			Type:     entities.GeoJSONType(domainGeo.Type),
			Bbox:     domainGeo.Bbox,
			Metadata: convertMetaData(domainGeo.Metadata),
			Features: utils.Map(domainGeo.Features, func(f domain.GeoJSONFeature) entities.GeoJSONFeature {
				return entities.GeoJSONFeature{
					Type:       entities.GeoJSONType(f.Type),
					Bbox:       f.Bbox,
					Properties: f.Properties,
					Geometry: entities.GeoJSONGeometry{
						Type:        entities.GeoJSONType(f.Geometry.Type),
						Coordinates: f.Geometry.Coordinates,
					},
				}
			}),
		})
	}
}

// @Summary		Download GPX file
// @Description	Downloads the GPX route file for a watering plan. Can be imported into GPS navigation devices.
// @Id				get-gpx-file
// @Tags			Watering Plan
// @Produce		application/gpx+xml
// @Success		200	{file}		application/gpx+xml
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/watering-plan/route/gpx/{gpx_name} [get]
// @Param			gpx_name	path	string	true	"GPX file name"
// @Security		Keycloak
func GetGpxFile(svc service.WateringPlanService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		objName := strings.Clone(c.Params("gpx_name"))

		fileStream, err := svc.GetGPXFileStream(ctx, objName)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		c.Set(fiber.HeaderContentType, "application/gpx+xml;charset=UTF-8")
		c.Set(fiber.HeaderContentDisposition, fmt.Sprintf("attachment; filename=%s", objName))
		_, err = io.Copy(c.Response().BodyWriter(), fileStream)
		return errorhandler.HandleError(err)
	}
}

func convertMetaData(domainMetadata domain.GeoJSONMetadata) entities.GeoJSONMetadata {
	return entities.GeoJSONMetadata{
		StartPoint: entities.GeoJSONLocation{
			Latitude:  domainMetadata.StartPoint.Latitude,
			Longitude: domainMetadata.StartPoint.Longitude,
		},
		EndPoint: entities.GeoJSONLocation{
			Latitude:  domainMetadata.EndPoint.Latitude,
			Longitude: domainMetadata.EndPoint.Longitude,
		},
		WateringPoint: entities.GeoJSONLocation{
			Latitude:  domainMetadata.WateringPoint.Latitude,
			Longitude: domainMetadata.WateringPoint.Longitude,
		},
	}
}
