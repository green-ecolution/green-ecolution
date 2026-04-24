package watering

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"time"

	"github.com/google/uuid"
	"github.com/spf13/viper"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	clusterDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/routing"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	userDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/user"
	vehicleDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
	wateringDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/watering"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

type WateringPlanService struct {
	wateringPlanRepo wateringDomain.WateringPlanRepository
	clusterRepo      clusterDomain.TreeClusterRepository
	vehicleRepo      vehicleDomain.VehicleRepository
	userRepo         userDomain.UserRepository
	routingRepo      routing.RoutingRepository
	gpxBucket        routing.S3Repository
	eventManager     *worker.EventManager
}

func NewWateringPlanService(
	wateringPlanRepository wateringDomain.WateringPlanRepository,
	clusterRepository clusterDomain.TreeClusterRepository,
	vehicleRepository vehicleDomain.VehicleRepository,
	userRepository userDomain.UserRepository,
	eventManager *worker.EventManager,
	routingRepo routing.RoutingRepository,
	gpxRepo routing.S3Repository,
) ports.WateringPlanService {
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

func (w *WateringPlanService) publishUpdateEvent(ctx context.Context, prevWp *wateringDomain.WateringPlan) error {
	log := logger.GetLogger(ctx)
	log.Debug("publish new event", "event", wateringDomain.EventTypeUpdate, "service", "WateringPlanService")
	updatedWp, err := w.GetByID(ctx, prevWp.ID)
	if err != nil {
		return err
	}
	event := wateringDomain.NewEventUpdate(prevWp, updatedWp)
	if err := w.eventManager.Publish(ctx, event); err != nil {
		log.Error("error while sending event after updating watering plan", "err", err, "watering_plan_id", prevWp.ID)
	}

	return nil
}

func (w *WateringPlanService) PreviewRoute(ctx context.Context, transporterID int32, trailerID *int32, clusterIDs []int32) (*routing.GeoJSON, error) {
	log := logger.GetLogger(ctx)
	transporter, err := w.vehicleRepo.GetByID(ctx, transporterID)
	if err != nil {
		log.Error("can't get selected transporter to preview route", "error", err, "vehicle_id", transporterID)
		return nil, ports.NewError(ports.NotFound, fmt.Sprintf("vehicle with id %d not found", transporterID))
	}

	var trailer *vehicleDomain.Vehicle
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
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	merged := vehicleDomain.MergeVehicles(transporter, trailer)
	coords := extractClusterCoordinates(clusters)

	geoJSON, err := w.routingRepo.GenerateRoute(ctx, merged.Height, merged.Width, merged.Length, merged.Weight, coords)
	if err != nil {
		if errors.Is(err, vehicleDomain.ErrUnknownType) {
			log.Debug("the vehicle type is not supported", "error", err, "vehicle_type", transporter.Type)
			return nil, ports.ErrVehicleUnsupportedType
		}
		log.Debug("failed to generate route", "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	return geoJSON, nil
}

func (w *WateringPlanService) GetAll(ctx context.Context, query shared.Query) ([]*wateringDomain.WateringPlan, int64, error) {
	log := logger.GetLogger(ctx)
	plans, totalCount, err := w.wateringPlanRepo.GetAll(ctx, query)
	if err != nil {
		log.Debug("failed to fetch watering plans", "error", err)
		return nil, 0, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return plans, totalCount, nil
}

func (w *WateringPlanService) GetByID(ctx context.Context, id int32) (*wateringDomain.WateringPlan, error) {
	log := logger.GetLogger(ctx)
	got, err := w.wateringPlanRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to fetch watering plan by id", "error", err, "watering_plan_id", id)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return got, nil
}

func (w *WateringPlanService) Create(ctx context.Context, createWp *wateringDomain.WateringPlanCreate) (*wateringDomain.WateringPlan, error) {
	log := logger.GetLogger(ctx)

	treeClusters, err := w.fetchTreeClusters(ctx, createWp.TreeClusterIDs)
	if err != nil {
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	transporter, err := w.vehicleRepo.GetByID(ctx, *createWp.TransporterID)
	if err != nil {
		log.Debug("failed to get transporter by id", "error", err, "transporter_id", *createWp.TransporterID)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	var trailer *vehicleDomain.Vehicle
	if createWp.TrailerID != nil {
		trailer, err = w.vehicleRepo.GetByID(ctx, *createWp.TrailerID)
		if err != nil {
			log.Debug("failed to get trailer by id", "error", err, "trailer_id", *createWp.TrailerID)
			return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
		}
	}

	if err := w.validateUsers(ctx, createWp.UserIDs, transporter, trailer); err != nil {
		log.Warn("selected user are not allowed to use this transporter and/or trailer", "error", err, "user_ids", createWp.UserIDs, "transporter_id", createWp.TransporterID, "trailer_id", createWp.TrailerID)
		return nil, err // err is already a service error
	}

	clusterIDs := utils.Map(treeClusters, func(c *clusterDomain.TreeCluster) int32 { return c.ID })
	totalTreeCount := countTotalTrees(treeClusters)

	created, err := w.wateringPlanRepo.Create(ctx, func(wp *wateringDomain.WateringPlan, _ wateringDomain.WateringPlanRepository) (bool, error) {
		wp.Date = createWp.Date
		wp.Description = createWp.Description
		wp.TransporterID = createWp.TransporterID
		wp.TrailerID = createWp.TrailerID
		wp.TreeClusterIDs = clusterIDs
		wp.UserIDs = createWp.UserIDs
		wp.TotalWaterRequired = utils.P(wateringDomain.CalculateRequiredWater(totalTreeCount))
		wp.Provider = createWp.Provider
		wp.AdditionalInfo = createWp.AdditionalInfo

		return true, nil
	})
	if err != nil {
		log.Debug("failed to create watering plan", "error", err)
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	mergedVehicle := vehicleDomain.MergeVehicles(transporter, trailer)
	coords := extractClusterCoordinates(treeClusters)
	treeCounts := extractTreeCounts(treeClusters)

	err = w.wateringPlanRepo.Update(ctx, created.ID, func(wp *wateringDomain.WateringPlan, _ wateringDomain.WateringPlanRepository) (bool, error) {
		gpxURL, err := w.getGpxRouteURL(ctx, created.ID, mergedVehicle.Height, mergedVehicle.Width, mergedVehicle.Length, mergedVehicle.Weight, coords)
		if err != nil {
			log.Warn("generating route in gpx fomat failed. will not save gpx route", "error", err, "watering_plan_id", created.ID)
		} else {
			wp.GpxURL = gpxURL
		}

		metadata, err := w.routingRepo.GenerateRouteInformation(ctx, mergedVehicle.Height, mergedVehicle.Width, mergedVehicle.Length, mergedVehicle.Weight, mergedVehicle.WaterCapacity, coords, treeCounts)
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
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	log.Info("watering plan created successfully", "watering_plan_id", created.ID)
	return created, nil
}

func (w *WateringPlanService) getGpxRouteURL(ctx context.Context, waterPlanID int32, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight float64, clusterCoordinates []shared.Coordinate) (string, error) {
	log := logger.GetLogger(ctx)
	r, err := w.routingRepo.GenerateRawGpxRoute(ctx, vehicleHeight, vehicleWidth, vehicleLength, vehicleWeight, clusterCoordinates)
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
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	return ioReader, nil
}

func (w *WateringPlanService) Update(ctx context.Context, id int32, updateWp *wateringDomain.WateringPlanUpdate) (*wateringDomain.WateringPlan, error) {
	log := logger.GetLogger(ctx)

	prevWp, err := w.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to get exitsting watering plan by id", "error", err, "watering_plan_id", id)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	treeClusters, err := w.fetchTreeClusters(ctx, updateWp.TreeClusterIDs)
	if err != nil {
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	transporter, err := w.vehicleRepo.GetByID(ctx, *updateWp.TransporterID)
	if err != nil {
		log.Debug("failed to get transporter by provided id", "error", err, "transporter_id", updateWp.TransporterID)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	var trailer *vehicleDomain.Vehicle
	if updateWp.TrailerID != nil {
		trailer, err = w.vehicleRepo.GetByID(ctx, *updateWp.TrailerID)
		if err != nil {
			log.Warn("failed to get trailer by provided id", "error", err, "trailer_id", updateWp.TrailerID)
			return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
		}
	}

	if err := w.validateUsers(ctx, updateWp.UserIDs, transporter, trailer); err != nil {
		return nil, err
	}

	clusterIDs := utils.Map(treeClusters, func(c *clusterDomain.TreeCluster) int32 { return c.ID })
	totalTreeCount := countTotalTrees(treeClusters)
	mergedVehicle := vehicleDomain.MergeVehicles(transporter, trailer)
	coords := extractClusterCoordinates(treeClusters)
	treeCounts := extractTreeCounts(treeClusters)

	err = w.wateringPlanRepo.Update(ctx, id, func(wp *wateringDomain.WateringPlan, _ wateringDomain.WateringPlanRepository) (bool, error) {
		wp.Date = updateWp.Date
		wp.Description = updateWp.Description
		wp.TransporterID = updateWp.TransporterID
		wp.TrailerID = updateWp.TrailerID
		wp.TreeClusterIDs = clusterIDs
		wp.Status = updateWp.Status
		wp.CancellationNote = updateWp.CancellationNote
		wp.Evaluation = updateWp.Evaluation
		wp.UserIDs = updateWp.UserIDs
		neededWater := wateringDomain.CalculateRequiredWater(totalTreeCount)
		wp.TotalWaterRequired = &neededWater
		wp.Provider = updateWp.Provider
		wp.AdditionalInfo = updateWp.AdditionalInfo

		if wp.ShouldRegenerateRoute(prevWp) {
			gpxURL, err := w.getGpxRouteURL(ctx, id, mergedVehicle.Height, mergedVehicle.Width, mergedVehicle.Length, mergedVehicle.Weight, coords)
			if err != nil {
				log.Warn("generating route in gpx fomat failed. will not save gpx route", "error", err, "watering_plan_id", id)
			} else {
				wp.GpxURL = gpxURL
			}
		}

		metadata, err := w.routingRepo.GenerateRouteInformation(ctx, mergedVehicle.Height, mergedVehicle.Width, mergedVehicle.Length, mergedVehicle.Weight, mergedVehicle.WaterCapacity, coords, treeCounts)
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
		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
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
		return ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	if err := w.wateringPlanRepo.Delete(ctx, id); err != nil {
		log.Debug("failed to delete watering plan", "error", err, "watering_plan_id", id)
		return ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	log.Info("watering plan deleted successfully", "watering_plan_id", id)
	return nil
}

func (w *WateringPlanService) Ready() bool {
	return w.wateringPlanRepo != nil
}

func (w *WateringPlanService) UpdateStatuses(ctx context.Context) error {
	log := logger.GetLogger(ctx)
	plans, _, err := w.wateringPlanRepo.GetAll(ctx, shared.Query{Provider: ""})
	if err != nil {
		log.Error("failed to fetch watering plans", "error", err)
		return err
	}

	cutoffTime := time.Now().Add(-24 * time.Hour) // 1 day ago
	for _, plan := range plans {
		if plan.IsExpired(cutoffTime) {
			err = w.wateringPlanRepo.Update(ctx, plan.ID, func(wp *wateringDomain.WateringPlan, _ wateringDomain.WateringPlanRepository) (bool, error) {
				wp.Status = wateringDomain.WateringPlanStatusNotCompeted
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
func (w *WateringPlanService) fetchTreeClusters(ctx context.Context, treeClusterIDs []*int32) ([]*clusterDomain.TreeCluster, error) {
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
		return nil, shared.ErrEntityNotFound("treecluster")
	}

	return clusters, nil
}

// returns service error
func (w *WateringPlanService) validateUsers(ctx context.Context, userIDs []*uuid.UUID, transporter, trailer *vehicleDomain.Vehicle) error {
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
		return ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	if len(users) == 0 {
		log.Debug("requested user ids in watering plan not found", "error", err, "user_ids", userIDStrings)
		return ports.MapError(ctx, shared.ErrEntityNotFound("users"), ports.ErrorLogEntityNotFound)
	}

	for _, u := range users {
		if !u.HasRole(userDomain.UserRoleTbz) {
			return ports.ErrUserNotCorrectRole
		}
	}

	var requiredLicenses []vehicleDomain.DrivingLicense
	if transporter != nil {
		requiredLicenses = append(requiredLicenses, transporter.DrivingLicense)
	}
	if trailer != nil {
		requiredLicenses = append(requiredLicenses, trailer.DrivingLicense)
	}

	hasQualifiedDriver := false
	for _, u := range users {
		if u.HasRequiredLicenses(requiredLicenses) {
			hasQualifiedDriver = true
			break
		}
	}
	if !hasQualifiedDriver {
		return ports.NewError(ports.BadRequest, "no user has all the required licenses")
	}

	return nil
}

func extractClusterCoordinates(clusters []*clusterDomain.TreeCluster) []shared.Coordinate {
	var coords []shared.Coordinate
	for _, c := range clusters {
		if c.Coordinate != nil {
			coords = append(coords, *c.Coordinate)
		}
	}
	return coords
}

func extractTreeCounts(clusters []*clusterDomain.TreeCluster) []int {
	counts := make([]int, len(clusters))
	for i, c := range clusters {
		counts[i] = len(c.TreeIDs)
	}
	return counts
}

func countTotalTrees(clusters []*clusterDomain.TreeCluster) int {
	total := 0
	for _, c := range clusters {
		total += len(c.TreeIDs)
	}
	return total
}
