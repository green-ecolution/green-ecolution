package storage

import (
	"context"
	"errors"
	"fmt"
	"io"

	"github.com/google/uuid"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
)

type ErrEntityNotFound string

func (e ErrEntityNotFound) Error() string {
	return fmt.Sprintf("entity not found: %s", string(e))
}

var (
	ErrIPNotFound            = errors.New("local ip not found")
	ErrIFacesNotFound        = errors.New("cant get interfaces")
	ErrIFacesAddressNotFound = errors.New("cant get interfaces address")
	ErrHostnameNotFound      = errors.New("cant get hostname")
	ErrCannotGetAppURL       = errors.New("cannot get app url")

	ErrIDNotFound           = errors.New("entity id not found")
	ErrIDAlreadyExists      = errors.New("entity id already exists")
	ErrSensorNotFound       = errors.New("sensor not found")
	ErrTreeClusterNotFound  = errors.New("treecluster not found")
	ErrRegionNotFound       = errors.New("region not found")
	ErrTreeNotFound         = errors.New("tree not found")
	ErrVehicleNotFound      = errors.New("vehicle not found")
	ErrWateringPlanNotFound = errors.New("watering plan not found")

	ErrUserNotFound           = errors.New("user not found")
	ErrUserNotCorrectRole     = errors.New("user has an incorrect role")
	ErrUserNotMatchingLicense = errors.New("user has no matching driving license")

	ErrUnknowError      = errors.New("unknown error")
	ErrToManyRows       = errors.New("receive more rows then expected")
	ErrConnectionClosed = errors.New("connection is closed")
	ErrTxClosed         = errors.New("transaction closed")
	ErrTxCommitRollback = errors.New("transaction cannot commit or rollback")

	ErrInvalidLatitude  = errors.New("latitude must be between 90,-90")
	ErrInvalidLongitude = errors.New("longitude must be between 180,-180")

	ErrUnknownVehicleType = errors.New("unknown vehicle type")
	ErrBucketNotExists    = errors.New("bucket don't exists")

	ErrPaginationValueInvalid = errors.New("pagination values are invalid")
	ErrInvalidMapConfig       = errors.New("map configuration not valid")

	ErrS3ServiceDisabled      = errors.New("s3 service is disabled")
	ErrAuthServiceDisabled    = errors.New("auth service is disabled")
	ErrRoutingServiceDisabled = errors.New("routing service is disabled")
)

type BasicCrudRepository[T entities.Entities] interface {
	// GetAll returns all entities
	GetAll(ctx context.Context) ([]*T, error)
	// GetByID returns one entity by id
	GetByID(ctx context.Context, id int32) (*T, error)
	// Create creates a new entity. It accepts a list of EntityFunc[T] to apply to the new entity
	Create(ctx context.Context, fn ...entities.EntityFunc[T]) (*T, error)
	// Update updates a already existing entity. It accepts a list of EntityFunc[T] to apply to the entity
	Update(ctx context.Context, id int32, fn ...entities.EntityFunc[T]) (*T, error)
	// Delete deletes a entity by id
	Delete(ctx context.Context, id int32) error
}

type InfoRepository interface {
	GetAppInfo(context.Context) (*entities.App, error)
}

type RegionRepository interface {
	// GetAll returns all regions
	GetAll(ctx context.Context) ([]*entities.Region, int64, error)
	// GetByID returns one region by id
	GetByID(ctx context.Context, id int32) (*entities.Region, error)
	// GetByPoint returns one region by latitude and longitude
	GetByPoint(ctx context.Context, latitude, longitude float64) (*entities.Region, error)
	// Create creates a new region. It accepts a list of functions to apply to the new region
	Create(ctx context.Context, fn ...entities.EntityFunc[entities.Region]) (*entities.Region, error)
	// Update updates a already existing region. It accepts a list of functions to apply to the region
	Update(ctx context.Context, id int32, fn ...entities.EntityFunc[entities.Region]) (*entities.Region, error)
	// Delete deletes a region by id
	Delete(ctx context.Context, id int32) error
}

type UserRepository interface {
	Create(ctx context.Context, user *entities.User, password string, roles []string) (*entities.User, error)
	RemoveSession(ctx context.Context, token string) error
	GetAll(ctx context.Context) ([]*entities.User, error)
	GetAllByRole(ctx context.Context, role entities.UserRole) ([]*entities.User, error)
	GetByIDs(ctx context.Context, ids []string) ([]*entities.User, error)
}

type VehicleRepository interface {
	// GetAll returns all vehicles that are not archived
	GetAll(ctx context.Context, query entities.Query) ([]*entities.Vehicle, int64, error)
	// GetAllWithArchived returns all vehicles and archived as well
	GetAllWithArchived(ctx context.Context, provider string) ([]*entities.Vehicle, int64, error)
	// GetAllByType returns all vehicles by vehicle type that are not archived
	GetAllByType(ctx context.Context, provider string, vehicleType entities.VehicleType) ([]*entities.Vehicle, int64, error)
	// GetAllByTypeWithArchived returns all vehicles by vehicle type and archived as well
	GetAllByTypeWithArchived(ctx context.Context, provider string, vehicleType entities.VehicleType) ([]*entities.Vehicle, int64, error)
	// GetAllArchived returns all archived vehicles
	GetAllArchived(ctx context.Context) ([]*entities.Vehicle, error)
	// GetAllWithWateringPlanCount retrieves all vehicles that are associated with at least one watering plan, along with the count of watering plans linked to each vehicle.
	GetAllWithWateringPlanCount(ctx context.Context) ([]*entities.VehicleEvaluation, error)
	// GetByID returns one vehicle by id
	GetByID(ctx context.Context, id int32) (*entities.Vehicle, error)
	// GetByPlate returns one vehicle by its plate
	GetByPlate(ctx context.Context, plate string) (*entities.Vehicle, error)
	// Create creates a new vehicle. It accepts a function that takes a vehicle that can be modified. Any changes made to the vehicle will be saved in the storage. If the function returns true, the vehicle will be created, otherwise it will not be created.
	Create(ctx context.Context, fn func(tc *entities.Vehicle, repo VehicleRepository) (bool, error)) (*entities.Vehicle, error)
	// Update updates a vehicle by id. It takes the id of the vehicle to update and a function that takes a vehicle that can be modified. Any changes made to the vehicle will be saved updated in the storage. If the function returns true, the vehicle will be updated, otherwise it will not be updated.
	Update(ctx context.Context, id int32, fn func(tc *entities.Vehicle, repo VehicleRepository) (bool, error)) error
	// Archive archives a vehicle by id
	Archive(ctx context.Context, id int32) error
	// Delete deletes a vehicle by id
	Delete(ctx context.Context, id int32) error
}

type WateringPlanRepository interface {
	// GetAll returns all watering plans
	GetAll(ctx context.Context, query entities.Query) ([]*entities.WateringPlan, int64, error)
	// GetCount returns count of all watering plans
	GetCount(ctx context.Context, query entities.Query) (int64, error)
	// GetByID returns one watering plan by id
	GetByID(ctx context.Context, id int32) (*entities.WateringPlan, error)
	// GetLinkedVehicleByIDAndType returnes all vehicles linked to a watering plan by the watering plan id and the vehicle type
	GetLinkedVehicleByIDAndType(ctx context.Context, id int32, vehicleType entities.VehicleType) (*entities.Vehicle, error)
	// GetLinkedTreeClustersByID retruns all tree cluster linked to a watering plan by the watering plan id
	GetLinkedTreeClustersByID(ctx context.Context, id int32) ([]*entities.TreeCluster, error)
	// GetLinkedUsersByID returns all linked user ids from relationship by a watering plan id
	GetLinkedUsersByID(ctx context.Context, id int32) ([]*uuid.UUID, error)
	// GetEvaluationValues returns all tree cluster relationship entities by a watering plan id
	GetEvaluationValues(ctx context.Context, id int32) ([]*entities.EvaluationValue, error)
	// GetTotalConsumedWater returns the total consumed water for all watering plans
	GetTotalConsumedWater(ctx context.Context) (int64, error)
	// GetAllUserCount returns count of all users linked to a watering plan
	GetAllUserCount(ctx context.Context) (int64, error)
	// Create creates a new watering plan. It accepts a function that takes a watering plan that can be modified. Any changes made to the plan will be saved in the storage. If the function returns true, the watering plan will be created, otherwise it will not be created.
	Create(ctx context.Context, fn func(tc *entities.WateringPlan, repo WateringPlanRepository) (bool, error)) (*entities.WateringPlan, error)
	// Update updates a watering plan by id. It takes the id of the watering plan to update and a function that takes a watering plan that can be modified. Any changes made to the plan will be saved updated in the storage. If the function returns true, the watering plan will be updated, otherwise it will not be updated.
	Update(ctx context.Context, id int32, fn func(tc *entities.WateringPlan, repo WateringPlanRepository) (bool, error)) error
	// Delete deletes a watering plan by id
	Delete(ctx context.Context, id int32) error
}

type TreeClusterRepository interface {
	// GetAll returns all tree clusters
	GetAll(ctx context.Context, query entities.TreeClusterQuery) ([]*entities.TreeCluster, int64, error)
	// GetCount returns all counts of tree cluster
	GetCount(ctx context.Context, query entities.TreeClusterQuery) (int64, error)
	// GetByID returns one tree cluster by id
	GetByID(ctx context.Context, id int32) (*entities.TreeCluster, error)
	// GetByIDs returns multiple tree cluster by ids
	// TODO: Add ability to optional preload
	GetByIDs(ctx context.Context, ids []int32) ([]*entities.TreeCluster, error)
	// Create creates a new tree cluster. It accepts a function that takes a tree cluster that can be modified. Any changes made to the tree cluster will be saved in the storage. If the function returns true, the tree cluster will be created, otherwise it will not be created.
	Create(ctx context.Context, fn func(tc *entities.TreeCluster, repo TreeClusterRepository) (bool, error)) (*entities.TreeCluster, error)
	// Update updates a tree cluster by id. It takes the id of the tree cluster to update and a function that takes a tree cluster that can be modified. Any changes made to the tree cluster will be saved updated in the storage. If the function returns true, the tree cluster will be updated, otherwise it will not be updated.
	Update(ctx context.Context, id int32, fn func(tc *entities.TreeCluster, repo TreeClusterRepository) (bool, error)) error
	// Delete deletes a tree cluster by id
	Delete(ctx context.Context, id int32) error
	// GetAllRegionsWithWateringPlanCount retrieves all tree cluster regions that are associated with at least one watering plan, along with the count of watering plans linked to each treecluster.
	GetAllRegionsWithWateringPlanCount(ctx context.Context) ([]*entities.RegionEvaluation, error)

	Archive(ctx context.Context, id int32) error
	LinkTreesToCluster(ctx context.Context, treeClusterID int32, treeIDs []int32) error
	GetCenterPoint(ctx context.Context, id int32) (float64, float64, error)
	GetAllLatestSensorDataByClusterID(ctx context.Context, tcID int32) ([]*entities.SensorData, error)
}

type TreeRepository interface {
	// GetAll returns all trees
	GetAll(ctx context.Context, query entities.TreeQuery) ([]*entities.Tree, int64, error)
	// GetCount returns count of all trees
	GetCount(ctx context.Context, query entities.TreeQuery) (int64, error)
	// GetByID returns one tree by id
	GetByID(ctx context.Context, id int32) (*entities.Tree, error)
	// Create creates a new tree. It accepts a function that takes a tree entity that can be modified. Any changes made to the tree will be saved in the storage. If the function returns true, the tree will be created, otherwise it will not be created.
	Create(ctx context.Context, fn func(tree *entities.Tree, repo TreeRepository) (bool, error)) (*entities.Tree, error)
	// Update updates an already existing tree by id. It takes the id of the tree to update and a function that takes a tree entity that can be modified. Any changes made to the tree will be saved in the storage. If the function returns true, the tree will be updated, otherwise it will not be updated.
	Update(ctx context.Context, id int32, updateFn func(tree *entities.Tree, repo TreeRepository) (bool, error)) (*entities.Tree, error)
	// Delete deletes a tree by id
	Delete(ctx context.Context, id int32) error

	GetByTreeClusterID(ctx context.Context, id int32) ([]*entities.Tree, error)
	GetSensorByTreeID(ctx context.Context, id int32) (*entities.Sensor, error)
	GetTreesByIDs(ctx context.Context, ids []int32) ([]*entities.Tree, error)
	GetByCoordinates(ctx context.Context, latitude, longitude float64) (*entities.Tree, error)
	GetBySensorID(ctx context.Context, id string) (*entities.Tree, error)
	GetBySensorIDs(ctx context.Context, ids ...string) ([]*entities.Tree, error)

	UnlinkTreeClusterID(ctx context.Context, treeClusterID int32) error
	UnlinkSensorID(ctx context.Context, sensorID string) error
	FindNearestTree(ctx context.Context, latitude, longitude float64) (*entities.Tree, error)
	GetDistinctPlantingYears(ctx context.Context) ([]int32, error)
}

type SensorRepository interface {
	GetAll(ctx context.Context, query entities.Query) ([]*entities.Sensor, int64, error)
	GetCount(ctx context.Context, query entities.Query) (int64, error)
	GetByID(ctx context.Context, id string) (*entities.Sensor, error)
	Create(ctx context.Context, createFn func(*entities.Sensor, SensorRepository) (bool, error)) (*entities.Sensor, error)
	Update(ctx context.Context, id string, updateFn func(*entities.Sensor, SensorRepository) (bool, error)) (*entities.Sensor, error)
	Delete(ctx context.Context, id string) error

	GetAllDataByID(ctx context.Context, id string) ([]*entities.SensorData, error)
	GetLatestSensorDataBySensorID(ctx context.Context, id string) (*entities.SensorData, error)
	InsertSensorData(ctx context.Context, data *entities.SensorData, id string) error
}

type RoutingRepository interface {
	GenerateRoute(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (*entities.GeoJSON, error)
	GenerateRawGpxRoute(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (io.ReadCloser, error)
	GenerateRouteInformation(ctx context.Context, vehicle *entities.Vehicle, clusters []*entities.TreeCluster) (*entities.RouteMetadata, error)
}

type S3Repository interface {
	BucketExists(ctx context.Context) (bool, error)
	// contentLength -1 => uploads to EOF
	PutObject(ctx context.Context, objName, contentType string, contentLength int64, r io.Reader) error
	GetObject(ctx context.Context, objName string) (io.ReadSeekCloser, error)
}

type AuthRepository interface {
	RetrospectToken(ctx context.Context, token string) (*entities.IntroSpectTokenResult, error)
	GetAccessTokenFromClientCode(ctx context.Context, code, redirectURL string) (*entities.ClientToken, error)
	RefreshToken(ctx context.Context, refreshToken string) (*entities.ClientToken, error)
	GetAccessTokenFromClientCredentials(ctx context.Context, clientID, clientSecret string) (*entities.ClientToken, error)
}

type Repository struct {
	Auth         AuthRepository
	Info         InfoRepository
	Sensor       SensorRepository
	Tree         TreeRepository
	User         UserRepository
	Vehicle      VehicleRepository
	TreeCluster  TreeClusterRepository
	Region       RegionRepository
	WateringPlan WateringPlanRepository
	Routing      RoutingRepository
	GpxBucket    S3Repository
}
