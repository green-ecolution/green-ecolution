package ports

import (
	"context"
	"errors"
	"io"
	"log/slog"
	"reflect"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/auth"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/evaluation"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/info"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/plugin"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/region"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/routing"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/user"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/watering"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

var (
	ErrIPNotFound            = errors.New("local ip not found")
	ErrIFacesNotFound        = errors.New("cant get interfaces")
	ErrIFacesAddressNotFound = errors.New("cant get interfaces address")
	ErrHostnameNotFound      = errors.New("cant get hostname")
	ErrValidation            = errors.New("validation error")

	ErrPluginRegistered       = NewError(BadRequest, "plugin already registered")
	ErrPluginNotRegistered    = NewError(BadRequest, "plugin not registered")
	ErrVehiclePlateTaken      = NewError(BadRequest, "number plate is already taken")
	ErrVehicleUnsupportedType = NewError(BadRequest, "vehicle type is not supported")
	ErrUserNotCorrectRole     = NewError(BadRequest, "user has an incorrect role")
)

type Error struct {
	Message string
	Code    ErrorCode
}

type ErrorLogMask int

// Bitmask
const (
	ErrorLogNothing        ErrorLogMask = -1
	ErrorLogAll            ErrorLogMask = 0
	ErrorLogEntityNotFound ErrorLogMask = (1 << iota)
	ErrorLogValidation
)

func NewError(code ErrorCode, msg string) Error {
	return Error{Code: code, Message: msg}
}

func (e Error) Error() string {
	return e.Message
}

func MapError(ctx context.Context, err error, errorMask ErrorLogMask) error {
	log := logger.GetLogger(ctx)
	var entityNotFoundErr shared.ErrEntityNotFound
	if errors.As(err, &entityNotFoundErr) {
		if errorMask&ErrorLogEntityNotFound == 0 {
			log.Error("can't find entity", "error", err)
		}
		return NewError(NotFound, entityNotFoundErr.Error())
	}

	if errors.Is(err, ErrValidation) {
		if errorMask&ErrorLogValidation == 0 {
			log.Error("failed to validate struct", "error", err)
		}
		return NewError(BadRequest, err.Error())
	}

	if errors.Is(err, shared.ErrS3ServiceDisabled) {
		log.Warn("s3 service is disabled")
		return NewError(Gone, err.Error())
	}

	if errors.Is(err, shared.ErrAuthServiceDisabled) {
		log.Warn("auth service is disabled")
		return NewError(Gone, err.Error())
	}

	if errors.Is(err, shared.ErrRoutingServiceDisabled) {
		log.Warn("routing service is disabled")
		return NewError(Gone, err.Error())
	}

	log.Error("an error has occurred", "error", err)
	return NewError(InternalError, err.Error())
}

type ErrorCode int

const (
	BadRequest    ErrorCode = 400
	Unauthorized  ErrorCode = 401
	Forbidden     ErrorCode = 403
	NotFound      ErrorCode = 404
	Conflict      ErrorCode = 409
	Gone          ErrorCode = 410
	InternalError ErrorCode = 500
)

type BasicCrudService[T any, CreateType any, UpdateType any] interface {
	GetAll(ctx context.Context, query shared.Query) ([]*T, int64, error)
	GetByID(ctx context.Context, id int32) (*T, error)
	Create(ctx context.Context, createData *CreateType) (*T, error)
	Update(ctx context.Context, id int32, updateData *UpdateType) (*T, error)
	Delete(ctx context.Context, id int32) error
}

type InfoService interface {
	Service
	GetAppInfo(context.Context) (*info.App, error)
	GetAppInfoResponse(context.Context) (*info.App, error)
	GetMapInfo(context.Context) (*info.Map, error)
	GetServerInfo(context.Context) (*info.Server, error)
	GetServices(context.Context) (*info.Services, error)
	GetStatistics(context.Context) (*info.DataStatistics, error)
}

type TreeService interface {
	Service
	GetAll(ctx context.Context, query tree.TreeQuery) ([]*tree.Tree, int64, error)
	GetByID(ctx context.Context, id int32) (*tree.Tree, error)
	Create(ctx context.Context, createData *tree.TreeCreate) (*tree.Tree, error)
	Update(ctx context.Context, id int32, updateData *tree.TreeUpdate) (*tree.Tree, error)
	Delete(ctx context.Context, id int32) error

	GetBySensorID(ctx context.Context, id sensor.SensorID) (*tree.Tree, error)
	GetNearestTrees(ctx context.Context, coord shared.Coordinate, limit int32) ([]*tree.TreeWithDistance, error)
	HandleNewSensorData(context.Context, *sensor.EventNewData) error
	UpdateWateringStatuses(ctx context.Context) error
	GetPlantingYears(ctx context.Context) ([]int32, error)
}

type EvaluationService interface {
	Service
	GetEvaluation(ctx context.Context) (*evaluation.Evaluation, error)
}

type AuthService interface {
	Service
	LoginRequest(ctx context.Context, loginRequest *auth.LoginRequest) *auth.LoginResp
	LogoutRequest(ctx context.Context, logoutRequest *auth.Logout) error
	ClientTokenCallback(ctx context.Context, loginCallback *auth.LoginCallback) (*auth.ClientToken, error)
	Register(ctx context.Context, user *user.RegisterUser) (*user.User, error)
	RetrospectToken(ctx context.Context, token string) (*auth.IntroSpectTokenResult, error)
	RefreshToken(ctx context.Context, refreshToken string) (*auth.ClientToken, error)
	GetAll(ctx context.Context) ([]*user.User, error)
	GetByIDs(ctx context.Context, ids []string) ([]*user.User, error)
	GetAllByRole(ctx context.Context, role user.UserRole) ([]*user.User, error)
}

type RegionService interface {
	Service
	GetAll(ctx context.Context) ([]*region.Region, int64, error)
	GetByID(ctx context.Context, id int32) (*region.Region, error)
}

type TreeClusterService interface {
	Service
	// TODO: use CrudService as soon as every service has pagination
	// CrudService[cluster.TreeCluster, cluster.TreeClusterCreate, cluster.TreeClusterUpdate]
	GetAll(ctx context.Context, query cluster.TreeClusterQuery) ([]*cluster.TreeCluster, int64, error)
	GetByID(ctx context.Context, id int32) (*cluster.TreeCluster, error)
	Create(ctx context.Context, createData *cluster.TreeClusterCreate) (*cluster.TreeCluster, error)
	Update(ctx context.Context, id int32, updateData *cluster.TreeClusterUpdate) (*cluster.TreeCluster, error)
	Delete(ctx context.Context, id int32) error

	HandleUpdateTree(context.Context, *tree.EventUpdate) error
	HandleCreateTree(context.Context, *tree.EventCreate) error
	HandleDeleteTree(context.Context, *tree.EventDelete) error
	HandleNewSensorData(context.Context, *sensor.EventNewData) error
	HandleUpdateWateringPlan(context.Context, *watering.EventUpdate) error
	UpdateWateringStatuses(ctx context.Context) error
}

type SensorService interface {
	Service
	GetAll(ctx context.Context, query shared.Query) ([]*sensor.Sensor, int64, error)
	GetByID(ctx context.Context, id sensor.SensorID) (*sensor.Sensor, error)
	Create(ctx context.Context, createData *sensor.SensorCreate) (*sensor.Sensor, error)
	Update(ctx context.Context, id sensor.SensorID, updateData *sensor.SensorUpdate) (*sensor.Sensor, error)
	Delete(ctx context.Context, id sensor.SensorID) error
	GetAllDataByID(ctx context.Context, id sensor.SensorID) ([]*sensor.SensorData, error)
	HandleMessage(ctx context.Context, payload *sensor.MqttPayload) (*sensor.SensorData, error)
	MapSensorToTree(ctx context.Context, sen *sensor.Sensor) error
	UpdateStatuses(ctx context.Context) error
}

type CrudService[T any, CreateType any, UpdateType any] interface {
	Service
	BasicCrudService[T, CreateType, UpdateType]
}

type VehicleService interface {
	Service
	GetAll(ctx context.Context, query vehicle.VehicleQuery) ([]*vehicle.Vehicle, int64, error)
	GetAllArchived(ctx context.Context) ([]*vehicle.Vehicle, int64, error)
	GetByID(ctx context.Context, id int32) (*vehicle.Vehicle, error)
	Create(ctx context.Context, createData *vehicle.VehicleCreate) (*vehicle.Vehicle, error)
	Update(ctx context.Context, id int32, updateData *vehicle.VehicleUpdate) (*vehicle.Vehicle, error)
	Delete(ctx context.Context, id int32) error
	Archive(ctx context.Context, id int32) error
	GetByPlate(ctx context.Context, plate string) (*vehicle.Vehicle, error)
}

type WateringPlanService interface {
	Service
	GetAll(ctx context.Context, query shared.Query) ([]*watering.WateringPlan, int64, error)
	GetByID(ctx context.Context, id int32) (*watering.WateringPlan, error)
	Create(ctx context.Context, createData *watering.WateringPlanCreate) (*watering.WateringPlan, error)
	Update(ctx context.Context, id int32, updateData *watering.WateringPlanUpdate) (*watering.WateringPlan, error)
	Delete(ctx context.Context, id int32) error

	PreviewRoute(ctx context.Context, transporterID int32, trailerID *int32, clusterIDs []int32) (*routing.GeoJSON, error)
	GetGPXFileStream(ctx context.Context, objName string) (io.ReadSeekCloser, error)

	UpdateStatuses(ctx context.Context) error
}

type PluginService interface {
	Service
	Register(ctx context.Context, plugin *plugin.Plugin) (*auth.ClientToken, error)
	RefreshToken(ctx context.Context, auth *plugin.AuthPlugin, slug string) (*auth.ClientToken, error)
	Get(ctx context.Context, slug string) (plugin.Plugin, error)
	GetAll(ctx context.Context) ([]plugin.Plugin, []time.Time)
	HeartBeat(ctx context.Context, slug string) error
	Unregister(ctx context.Context, slug string)
	StartCleanup(ctx context.Context)
}

type Service interface {
	Ready() bool
}

type Services struct {
	InfoService         InfoService
	TreeService         TreeService
	AuthService         AuthService
	RegionService       RegionService
	TreeClusterService  TreeClusterService
	SensorService       SensorService
	VehicleService      VehicleService
	PluginService       PluginService
	WateringPlanService WateringPlanService
	EvaluationService   EvaluationService
}

type ServicesInterface interface {
	AllServicesReady() bool
}

func (s *Services) AllServicesReady() bool {
	v := reflect.ValueOf(s).Elem()
	for i := 0; i < v.NumField(); i++ {
		service := v.Field(i).Interface()
		if srv, ok := service.(Service); ok {
			if !srv.Ready() {
				return false
			}
		} else {
			slog.Debug("Service does not implement the Service interface", "service", v.Field(i).Type().Name())
			return false
		}
	}
	return true
}
