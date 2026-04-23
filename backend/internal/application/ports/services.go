package ports

import (
	"context"
	"errors"
	"io"
	"log/slog"
	"reflect"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
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
	GetAppInfo(context.Context) (*shared.App, error)
	GetAppInfoResponse(context.Context) (*shared.App, error)
	GetMapInfo(context.Context) (*shared.Map, error)
	GetServerInfo(context.Context) (*shared.Server, error)
	GetServices(context.Context) (*shared.Services, error)
	GetStatistics(context.Context) (*shared.DataStatistics, error)
}

type TreeService interface {
	Service
	GetAll(ctx context.Context, query shared.TreeQuery) ([]*shared.Tree, int64, error)
	GetByID(ctx context.Context, id int32) (*shared.Tree, error)
	Create(ctx context.Context, createData *shared.TreeCreate) (*shared.Tree, error)
	Update(ctx context.Context, id int32, updateData *shared.TreeUpdate) (*shared.Tree, error)
	Delete(ctx context.Context, id int32) error

	GetBySensorID(ctx context.Context, id shared.SensorID) (*shared.Tree, error)
	GetNearestTrees(ctx context.Context, coord shared.Coordinate, limit int32) ([]*shared.TreeWithDistance, error)
	HandleNewSensorData(context.Context, *shared.EventNewSensorData) error
	UpdateWateringStatuses(ctx context.Context) error
	GetPlantingYears(ctx context.Context) ([]int32, error)
}

type EvaluationService interface {
	Service
	GetEvaluation(ctx context.Context) (*shared.Evaluation, error)
}

type AuthService interface {
	Service
	LoginRequest(ctx context.Context, loginRequest *shared.LoginRequest) *shared.LoginResp
	LogoutRequest(ctx context.Context, logoutRequest *shared.Logout) error
	ClientTokenCallback(ctx context.Context, loginCallback *shared.LoginCallback) (*shared.ClientToken, error)
	Register(ctx context.Context, user *shared.RegisterUser) (*shared.User, error)
	RetrospectToken(ctx context.Context, token string) (*shared.IntroSpectTokenResult, error)
	RefreshToken(ctx context.Context, refreshToken string) (*shared.ClientToken, error)
	GetAll(ctx context.Context) ([]*shared.User, error)
	GetByIDs(ctx context.Context, ids []string) ([]*shared.User, error)
	GetAllByRole(ctx context.Context, role shared.UserRole) ([]*shared.User, error)
}

type RegionService interface {
	Service
	GetAll(ctx context.Context) ([]*shared.Region, int64, error)
	GetByID(ctx context.Context, id int32) (*shared.Region, error)
}

type TreeClusterService interface {
	Service
	// TODO: use CrudService as soon as every service has pagination
	// CrudService[shared.TreeCluster, shared.TreeClusterCreate, shared.TreeClusterUpdate]
	GetAll(ctx context.Context, query shared.TreeClusterQuery) ([]*shared.TreeCluster, int64, error)
	GetByID(ctx context.Context, id int32) (*shared.TreeCluster, error)
	Create(ctx context.Context, createData *shared.TreeClusterCreate) (*shared.TreeCluster, error)
	Update(ctx context.Context, id int32, updateData *shared.TreeClusterUpdate) (*shared.TreeCluster, error)
	Delete(ctx context.Context, id int32) error

	HandleUpdateTree(context.Context, *shared.EventUpdateTree) error
	HandleCreateTree(context.Context, *shared.EventCreateTree) error
	HandleDeleteTree(context.Context, *shared.EventDeleteTree) error
	HandleNewSensorData(context.Context, *shared.EventNewSensorData) error
	HandleUpdateWateringPlan(context.Context, *shared.EventUpdateWateringPlan) error
	UpdateWateringStatuses(ctx context.Context) error
}

type SensorService interface {
	Service
	GetAll(ctx context.Context, query shared.Query) ([]*shared.Sensor, int64, error)
	GetByID(ctx context.Context, id shared.SensorID) (*shared.Sensor, error)
	Create(ctx context.Context, createData *shared.SensorCreate) (*shared.Sensor, error)
	Update(ctx context.Context, id shared.SensorID, updateData *shared.SensorUpdate) (*shared.Sensor, error)
	Delete(ctx context.Context, id shared.SensorID) error
	GetAllDataByID(ctx context.Context, id shared.SensorID) ([]*shared.SensorData, error)
	HandleMessage(ctx context.Context, payload *shared.MqttPayload) (*shared.SensorData, error)
	MapSensorToTree(ctx context.Context, sen *shared.Sensor) error
	UpdateStatuses(ctx context.Context) error
}

type CrudService[T any, CreateType any, UpdateType any] interface {
	Service
	BasicCrudService[T, CreateType, UpdateType]
}

type VehicleService interface {
	Service
	GetAll(ctx context.Context, query shared.VehicleQuery) ([]*shared.Vehicle, int64, error)
	GetAllArchived(ctx context.Context) ([]*shared.Vehicle, error)
	GetByID(ctx context.Context, id int32) (*shared.Vehicle, error)
	Create(ctx context.Context, createData *shared.VehicleCreate) (*shared.Vehicle, error)
	Update(ctx context.Context, id int32, updateData *shared.VehicleUpdate) (*shared.Vehicle, error)
	Delete(ctx context.Context, id int32) error
	Archive(ctx context.Context, id int32) error
	GetByPlate(ctx context.Context, plate string) (*shared.Vehicle, error)
}

type WateringPlanService interface {
	Service
	GetAll(ctx context.Context, query shared.Query) ([]*shared.WateringPlan, int64, error)
	GetByID(ctx context.Context, id int32) (*shared.WateringPlan, error)
	Create(ctx context.Context, createData *shared.WateringPlanCreate) (*shared.WateringPlan, error)
	Update(ctx context.Context, id int32, updateData *shared.WateringPlanUpdate) (*shared.WateringPlan, error)
	Delete(ctx context.Context, id int32) error

	PreviewRoute(ctx context.Context, transporterID int32, trailerID *int32, clusterIDs []int32) (*shared.GeoJSON, error)
	GetGPXFileStream(ctx context.Context, objName string) (io.ReadSeekCloser, error)

	UpdateStatuses(ctx context.Context) error
}

type PluginService interface {
	Service
	Register(ctx context.Context, plugin *shared.Plugin) (*shared.ClientToken, error)
	RefreshToken(ctx context.Context, auth *shared.AuthPlugin, slug string) (*shared.ClientToken, error)
	Get(ctx context.Context, slug string) (shared.Plugin, error)
	GetAll(ctx context.Context) ([]shared.Plugin, []time.Time)
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
