package wateringplan

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"time"

	"github.com/google/uuid"
	"github.com/spf13/viper"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

type WateringPlanService struct {
	wateringPlanRepo storage.WateringPlanRepository
	clusterRepo      storage.TreeClusterRepository
	vehicleRepo      storage.VehicleRepository
	userRepo         storage.UserRepository
	routingRepo      storage.RoutingRepository
	gpxBucket        storage.S3Repository
	eventManager     *worker.EventManager
}

func NewWateringPlanService(
	wateringPlanRepository storage.WateringPlanRepository,
	clusterRepository storage.TreeClusterRepository,
	vehicleRepository storage.VehicleRepository,
	userRepository storage.UserRepository,
	eventManager *worker.EventManager,
	routingRepo storage.RoutingRepository,
	gpxRepo storage.S3Repository,
) service.WateringPlanService {
	return &WateringPlanService{
		wateringPlanRepo: wateringPlanRepository,
		clusterRepo:      clusterRepository,
		vehicleRepo:      vehicleRepository,
		userRepo:         userRepository,
		routingRepo:      routingRepo,
		gpxBucket:        gpxRepo,
		eventManager:     eventManager,
	}
}

func (w *WateringPlanService) publishUpdateEvent(ctx context.Context, prevWp *entities.WateringPlan) error {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", entities.EventTypeUpdateWateringPlan, "service", "WateringPlanService")
	updatedWp, err := w.GetByID(ctx, prevWp.ID)
	if err != nil {
		return err
	}
	event := entities.NewEventUpdateWateringPlan(prevWp, updatedWp)
	if err := w.eventManager.Publish(ctx, event); err != nil {
		log.Error("error while sending event after updating watering plan", "err", err, "watering_plan_id", prevWp.ID)
	}

	return nil
}

func (w *WateringPlanService) PreviewRoute(ctx context.Context, transporterID int32, trailerID *int32, clusterIDs []int32) (*entities.GeoJSON, error) {
	log := logger.GetLogger(ctx)
	transporter, err := w.vehicleRepo.GetByID(ctx, transporterID)
	if err != nil {
		log.Error("can't get selected transporter to preview route", "error", err, "vehicle_id", transporterID)
		return nil, service.NewError(service.NotFound, fmt.Sprintf("vehicle with id %d not found", transporterID))
	}

	var trailer *entities.Vehicle
	if trailerID != nil {
		trailer, err = w.vehicleRepo.GetByID(ctx, *trailerID)
		if err != nil {
			log.Warn("can't get selected trailer to preview route. route will be calculated without trailer", "error", err, "trailer_id", trailerID)
		}
	}

	clusters, err := w.clusterRepo.GetByIDs(ctx, clusterIDs)
	if err != nil {
		// when error, something is wrong with the db, else clusters should be an empty array
		log.Debug("failed to get cluster by provided ids", "cluster_ids", clusterIDs)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	geoJSON, err := w.routingRepo.GenerateRoute(ctx, entities.MergeVehicles(transporter, trailer), clusters)
	if err != nil {
		if errors.Is(err, storage.ErrUnknownVehicleType) {
			log.Debug("the vehicle type is not supported", "error", err, "vehicle_type", transporter.Type)
			return nil, service.ErrVehicleUnsupportedType
		}
		log.Debug("failed to generate route", "error", err)
		return nil, service.MapError(ctx, err, service.ErrorLogAll)
	}

	return geoJSON, nil
}

func (w *WateringPlanService) GetAll(ctx context.Context, query entities.Query) ([]*entities.WateringPlan, int64, error) {
	log := logger.GetLogger(ctx)
	plans, totalCount, err := w.wateringPlanRepo.GetAll(ctx, query)
	if err != nil {
		log.Debug("failed to fetch watering plans", "error", err)
		return nil, 0, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	return plans, totalCount, nil
}

func (w *WateringPlanService) GetByID(ctx context.Context, id int32) (*entities.WateringPlan, error) {
	log := logger.GetLogger(ctx)
	got, err := w.wateringPlanRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to fetch watering plan by id", "error", err, "watering_plan_id", id)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	return got, nil
}

func (w *WateringPlanService) Create(ctx context.Context, createWp *entities.WateringPlanCreate) (*entities.WateringPlan, error) {
	log := logger.GetLogger(ctx)

	treeClusters, err := w.fetchTreeClusters(ctx, createWp.TreeClusterIDs)
	if err != nil {
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	transporter, err := w.vehicleRepo.GetByID(ctx, *createWp.TransporterID)
	if err != nil {
		log.Debug("failed to get transporter by id", "error", err, "transporter_id", *createWp.TransporterID)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	var trailer *entities.Vehicle
	if createWp.TrailerID != nil {
		trailer, err = w.vehicleRepo.GetByID(ctx, *createWp.TrailerID)
		if err != nil {
			log.Debug("failed to get trailer by id", "error", err, "trailer_id", *createWp.TrailerID)
			return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
		}
	}

	if err := w.validateUsers(ctx, createWp.UserIDs, transporter, trailer); err != nil {
		log.Warn("selected user are not allowed to use this transporter and/or trailer", "error", err, "user_ids", createWp.UserIDs, "transporter_id", createWp.TransporterID, "trailer_id", createWp.TrailerID)
		return nil, err // err is already a service error
	}

	created, err := w.wateringPlanRepo.Create(ctx, func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
		wp.Date = createWp.Date
		wp.Description = createWp.Description
		wp.Transporter = transporter
		wp.Trailer = trailer
		wp.TreeClusters = treeClusters
		wp.UserIDs = createWp.UserIDs
		wp.TotalWaterRequired = utils.P(wp.CalculateRequiredWater())
		wp.Provider = createWp.Provider
		wp.AdditionalInfo = createWp.AdditionalInfo

		return true, nil
	})
	if err != nil {
		log.Debug("failed to create watering plan", "error", err)
		return nil, service.MapError(ctx, err, service.ErrorLogAll)
	}

	err = w.wateringPlanRepo.Update(ctx, created.ID, func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
		mergedVehicle := entities.MergeVehicles(transporter, trailer)
		gpxURL, err := w.getGpxRouteURL(ctx, created.ID, mergedVehicle, treeClusters)
		if err != nil {
			log.Warn("generating route in gpx fomat failed. will not save gpx route", "error", err, "watering_plan_id", created.ID)
		} else {
			wp.GpxURL = gpxURL
		}

		metadata, err := w.routingRepo.GenerateRouteInformation(ctx, mergedVehicle, treeClusters)
		if err != nil {
			log.Warn("generating route information failed. will not save route metadata", "error", err, "watering_plan_id", created.ID)
		} else {
			wp.Distance = utils.P(metadata.Distance)
			wp.Duration = metadata.Time
			wp.RefillCount = metadata.Refills
		}

		return true, nil
	})

	if err != nil {
		log.Debug("failed to apply generate gpx url and route metadata to recently created watering plan", "error", err, "watering_plan_id", created.ID)
		return nil, service.MapError(ctx, err, service.ErrorLogAll)
	}

	log.Info("watering plan created successfully", "watering_plan_id", created.ID)
	return created, nil
}

func (w *WateringPlanService) getGpxRouteURL(ctx context.Context, waterPlanID int32, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (string, error) {
	log := logger.GetLogger(ctx)
	r, err := w.routingRepo.GenerateRawGpxRoute(ctx, vehicle, clusters)
	if err != nil {
		log.Error("failed to generate gpx route", "error", err)
		return "", err
	}
	defer r.Close()

	objName := fmt.Sprintf("waterplan-%d.gpx", waterPlanID)

	var buf bytes.Buffer
	length, err := io.Copy(&buf, r)
	if err != nil {
		log.Error("error while reading gpx response", "error", err)
		return "", err
	}

	if err := w.gpxBucket.PutObject(ctx, objName, "application/gpx+xml;charset=UTF-8 ", length, &buf); err != nil {
		log.Error("failed to upload gpx object to bucket", "error", err, "bucket_name", viper.GetString("s3.route-gpx.bucket"), "obj_name", objName)
		return "", err
	}

	log.Info("gpx route successfully uploaded to s3 bucket", "obj_name", objName, "bucket_name", viper.GetString("s3.route-gpx.bucket"))
	return fmt.Sprintf("/v1/watering-plan/route/gpx/%s", objName), nil
}

func (w *WateringPlanService) GetGPXFileStream(ctx context.Context, objName string) (io.ReadSeekCloser, error) {
	log := logger.GetLogger(ctx)
	log.Debug("get gpx route object from bucket", "obj_name", objName, "bucket_name", viper.GetString("s3.route-gpx.bucket"))
	ioReader, err := w.gpxBucket.GetObject(ctx, objName)
	if err != nil {
		return nil, service.MapError(ctx, err, service.ErrorLogAll)
	}

	return ioReader, nil
}

func (w *WateringPlanService) Update(ctx context.Context, id int32, updateWp *entities.WateringPlanUpdate) (*entities.WateringPlan, error) {
	log := logger.GetLogger(ctx)

	prevWp, err := w.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to get exitsting watering plan by id", "error", err, "watering_plan_id", id)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	treeClusters, err := w.fetchTreeClusters(ctx, updateWp.TreeClusterIDs)
	if err != nil {
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	transporter, err := w.vehicleRepo.GetByID(ctx, *updateWp.TransporterID)
	if err != nil {
		log.Debug("failed to get transporter by provided id", "error", err, "transporter_id", updateWp.TransporterID)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	var trailer *entities.Vehicle
	if updateWp.TrailerID != nil {
		trailer, err = w.vehicleRepo.GetByID(ctx, *updateWp.TrailerID)
		if err != nil {
			log.Warn("failed to get trailer by provided id", "error", err, "trailer_id", updateWp.TrailerID)
			return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
		}
	}

	if err := w.validateUsers(ctx, updateWp.UserIDs, transporter, trailer); err != nil {
		return nil, err
	}

	err = w.wateringPlanRepo.Update(ctx, id, func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
		wp.Date = updateWp.Date
		wp.Description = updateWp.Description
		wp.Transporter = transporter
		wp.Trailer = trailer
		wp.TreeClusters = treeClusters
		wp.Status = updateWp.Status
		wp.CancellationNote = updateWp.CancellationNote
		wp.Evaluation = updateWp.Evaluation
		wp.UserIDs = updateWp.UserIDs
		neededWater := wp.CalculateRequiredWater()
		wp.TotalWaterRequired = &neededWater
		wp.Provider = updateWp.Provider
		wp.AdditionalInfo = updateWp.AdditionalInfo

		mergedVehicle := entities.MergeVehicles(transporter, trailer)
		if wp.ShouldRegenerateRoute(prevWp) {
			gpxURL, err := w.getGpxRouteURL(ctx, id, mergedVehicle, treeClusters)
			if err != nil {
				log.Warn("generating route in gpx fomat failed. will not save gpx route", "error", err, "watering_plan_id", id)
			} else {
				wp.GpxURL = gpxURL
			}
		}

		metadata, err := w.routingRepo.GenerateRouteInformation(ctx, mergedVehicle, treeClusters)
		if err != nil {
			log.Warn("generating route information failed. will not route metadata", "error", err, "watering_plan_id", id)
		} else {
			wp.Distance = utils.P(metadata.Distance)
			wp.Duration = metadata.Time
			wp.RefillCount = metadata.Refills
		}

		return true, nil
	})

	if err != nil {
		log.Debug("failed to update watering plan", "error", err, "watering_plan_id", id)
		return nil, service.MapError(ctx, err, service.ErrorLogAll)
	}

	log.Info("watering plan updated successfully", "watering_plan_id", id)
	if err := w.publishUpdateEvent(ctx, prevWp); err != nil {
		log.Warn("failed to publish update event", "error", err)
	}
	return w.GetByID(ctx, id)
}

func (w *WateringPlanService) Delete(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)
	if _, err := w.wateringPlanRepo.GetByID(ctx, id); err != nil {
		return service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	if err := w.wateringPlanRepo.Delete(ctx, id); err != nil {
		log.Debug("failed to delete watering plan", "error", err, "watering_plan_id", id)
		return service.MapError(ctx, err, service.ErrorLogAll)
	}

	log.Info("watering plan deleted successfully", "watering_plan_id", id)
	return nil
}

func (w *WateringPlanService) Ready() bool {
	return w.wateringPlanRepo != nil
}

func (w *WateringPlanService) UpdateStatuses(ctx context.Context) error {
	log := logger.GetLogger(ctx)
	plans, _, err := w.wateringPlanRepo.GetAll(ctx, entities.Query{Provider: ""})
	if err != nil {
		log.Error("failed to fetch watering plans", "error", err)
		return err
	}

	cutoffTime := time.Now().Add(-24 * time.Hour) // 1 day ago
	for _, plan := range plans {
		if plan.IsExpired(cutoffTime) {
			err = w.wateringPlanRepo.Update(ctx, plan.ID, func(wp *entities.WateringPlan, _ storage.WateringPlanRepository) (bool, error) {
				wp.Status = entities.WateringPlanStatusNotCompeted
				return true, nil
			})

			if err != nil {
				log.Error("failed to update watering plan status to »not competed«", "watering_plan_id", plan.ID, "error", err)
			} else {
				log.Debug("watering plan marked as »not competed«", "watering_plan_id", plan.ID)
			}
		}
	}

	log.Info("watering plan status update process completed successfully")
	return nil
}

// returns service error
func (w *WateringPlanService) fetchTreeClusters(ctx context.Context, treeClusterIDs []*int32) ([]*entities.TreeCluster, error) {
	log := logger.GetLogger(ctx)
	clusters, err := w.clusterRepo.GetByIDs(ctx, utils.Map(treeClusterIDs, func(cID *int32) int32 {
		return *cID
	}))
	if err != nil {
		log.Debug("failed to fetch tree cluster specified by requested ids", "cluster_ids", treeClusterIDs, "error", err)
		return nil, err
	}

	if len(clusters) == 0 {
		log.Debug("requested tree cluster ids in watering plan are not found", "cluster_ids", treeClusterIDs, "error", err)
		return nil, storage.ErrEntityNotFound("treecluster")
	}

	return clusters, nil
}

// returns service error
func (w *WateringPlanService) validateUsers(ctx context.Context, userIDs []*uuid.UUID, transporter, trailer *entities.Vehicle) error {
	log := logger.GetLogger(ctx)
	var userIDStrings []string
	for _, id := range userIDs {
		if id != nil {
			userIDStrings = append(userIDStrings, utils.UUIDToString(*id))
		}
	}

	// Checks if the incoming user ids are belonging to valid users
	users, err := w.userRepo.GetByIDs(ctx, userIDStrings)
	if err != nil {
		log.Debug("failed to fetch users by id", "error", err, "user_ids", userIDStrings)
		return service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	if len(users) == 0 {
		log.Debug("requested user ids in watering plan not found", "error", err, "user_ids", userIDStrings)
		return service.MapError(ctx, storage.ErrEntityNotFound("users"), service.ErrorLogEntityNotFound)
	}

	for _, user := range users {
		if !user.HasRole(entities.UserRoleTbz) {
			return service.ErrUserNotCorrectRole
		}
	}

	var requiredLicenses []entities.DrivingLicense
	if transporter != nil {
		requiredLicenses = append(requiredLicenses, transporter.DrivingLicense)
	}
	if trailer != nil {
		requiredLicenses = append(requiredLicenses, trailer.DrivingLicense)
	}

	hasQualifiedDriver := false
	for _, user := range users {
		if user.HasRequiredLicenses(requiredLicenses) {
			hasQualifiedDriver = true
			break
		}
	}
	if !hasQualifiedDriver {
		return service.NewError(service.BadRequest, "no user has all the required licenses")
	}

	return nil
}
