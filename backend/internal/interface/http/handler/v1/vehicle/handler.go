package vehicle

import (
	"strconv"
	"strings"

	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities/mapper"
	handler "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/handler/v1"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/handler/v1/errorhandler"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/pagination"
)

// @Summary		Get all vehicles
// @Description	Retrieves a paginated list of all vehicles. Supports filtering by type, provider, and archive status.
// @Id				get-all-vehicles
// @Tags			Vehicle
// @Produce		json
// @Success		200	{object}	entities.VehicleListResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/vehicle [get]
// @Param			page		query	int		false	"Page number for pagination"
// @Param			limit		query	int		false	"Number of items per page"
// @Param			type		query	string	false	"Filter by vehicle type"
// @Param			provider	query	string	false	"Filter by data provider"
// @Param			archived	query	bool	false	"Include archived vehicles"
// @Security		Keycloak
func GetAllVehicles(svc ports.VehicleService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()

		var domainData []*vehicle.Vehicle
		var totalCount int64
		var err error
		var query vehicle.VehicleQuery

		if err := c.QueryParser(&query); err != nil {
			return errorhandler.HandleError(err)
		}

		domainData, totalCount, err = svc.GetAll(ctx, query)

		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := make([]*entities.VehicleResponse, len(domainData))
		for i, domain := range domainData {
			data[i] = mapper.VehicleFromResponse(domain)
		}

		return c.JSON(entities.VehicleListResponse{
			Data:       data,
			Pagination: pagination.Create(ctx, totalCount),
		})
	}
}

// @Summary		Get vehicle by ID
// @Description	Retrieves detailed information about a specific vehicle including its type and water capacity.
// @Id				get-vehicle-by-id
// @Tags			Vehicle
// @Produce		json
// @Success		200	{object}	entities.VehicleResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/vehicle/{id} [get]
// @Param			id	path	int	true	"Vehicle ID"
// @Security		Keycloak
func GetVehicleByID(svc ports.VehicleService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("id"))
		if err != nil {
			err := ports.NewError(ports.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		domainData, err := svc.GetByID(ctx, int32(id))

		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(mapper.VehicleFromResponse(domainData))
	}
}

// @Summary		Get vehicle by plate
// @Description	Retrieves a vehicle by its license plate number.
// @Id				get-vehicle-by-plate
// @Tags			Vehicle
// @Produce		json
// @Success		200	{object}	entities.VehicleResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/vehicle/plate/{plate} [get]
// @Param			plate	path	string	true	"Vehicle license plate number"
// @Security		Keycloak
func GetVehicleByPlate(svc ports.VehicleService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()

		plate := strings.Clone(c.Params("plate"))
		if plate == "" {
			err := ports.NewError(ports.BadRequest, "invalid Plate format")
			return errorhandler.HandleError(err)
		}

		domainData, err := svc.GetByPlate(ctx, plate)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(mapper.VehicleFromResponse(domainData))
	}
}

// @Summary		Create vehicle
// @Description	Creates a new vehicle with the provided data including type, plate number, and water capacity.
// @Id				create-vehicle
// @Tags			Vehicle
// @Accept			json
// @Produce		json
// @Success		201	{object}	entities.VehicleResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/vehicle [post]
// @Param			body	body	entities.VehicleCreateRequest	true	"Vehicle data to create"
// @Security		Keycloak
func CreateVehicle(svc ports.VehicleService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()

		req, err := handler.BindAndValidate[entities.VehicleCreateRequest](c)
		if err != nil {
			return err
		}

		domainReq, err := mapper.VehicleFromCreateRequest(req)
		if err != nil {
			return errorhandler.HandleError(ports.NewError(ports.BadRequest, err.Error()))
		}
		domainData, err := svc.Create(ctx, domainReq)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		data := mapper.VehicleFromResponse(domainData)
		return c.Status(fiber.StatusCreated).JSON(data)
	}
}

// @Summary		Update vehicle
// @Description	Updates an existing vehicle with the provided data.
// @Id				update-vehicle
// @Tags			Vehicle
// @Accept			json
// @Produce		json
// @Success		200	{object}	entities.VehicleResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/vehicle/{id} [put]
// @Param			id		path	int								true	"Vehicle ID"
// @Param			body	body	entities.VehicleUpdateRequest	true	"Vehicle data to update"
// @Security		Keycloak
func UpdateVehicle(svc ports.VehicleService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("id"))
		if err != nil {
			err := ports.NewError(ports.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		req, err := handler.BindAndValidate[entities.VehicleUpdateRequest](c)
		if err != nil {
			return err
		}

		domainReq, err := mapper.VehicleFromUpdateRequest(req)
		if err != nil {
			return errorhandler.HandleError(ports.NewError(ports.BadRequest, err.Error()))
		}
		domainData, err := svc.Update(ctx, int32(id), domainReq)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(mapper.VehicleFromResponse(domainData))
	}
}

// @Summary		Get archived vehicles
// @Description	Retrieves a list of all archived vehicles.
// @Id				get-archived-vehicles
// @Tags			Vehicle
// @Produce		json
// @Success		200	{object}	[]entities.VehicleResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/vehicle/archive [get]
// @Security		Keycloak
func GetArchiveVehicles(svc ports.VehicleService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		v, _, err := svc.GetAllArchived(ctx)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.JSON(mapper.VehicleFromResponseList(v))
	}
}

// @Summary		Archive vehicle
// @Description	Archives a vehicle. Archived vehicles are hidden from the default list but can still be retrieved. Returns 409 if vehicle is in use by active watering plans.
// @Id				archive-vehicle
// @Tags			Vehicle
// @Produce		json
// @Success		204
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		409	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/vehicle/archive/{id} [post]
// @Param			id	path	int	true	"Vehicle ID"
// @Security		Keycloak
func ArchiveVehicle(svc ports.VehicleService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("id"))
		if err != nil {
			err := ports.NewError(ports.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		err = svc.Archive(ctx, int32(id))
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.SendStatus(fiber.StatusNoContent)
	}
}

// @Summary		Delete vehicle
// @Description	Permanently deletes a vehicle. Consider archiving instead if the vehicle might be needed for historical records.
// @Id				delete-vehicle
// @Tags			Vehicle
// @Produce		json
// @Success		204
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/vehicle/{id} [delete]
// @Param			id	path	int	true	"Vehicle ID"
// @Security		Keycloak
func DeleteVehicle(svc ports.VehicleService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		id, err := strconv.Atoi(c.Params("id"))
		if err != nil {
			err := ports.NewError(ports.BadRequest, "invalid ID format")
			return errorhandler.HandleError(err)
		}

		err = svc.Delete(ctx, int32(id))
		if err != nil {
			return errorhandler.HandleError(err)
		}

		return c.SendStatus(fiber.StatusNoContent)
	}
}
