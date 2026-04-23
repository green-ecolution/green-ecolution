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
	var entityNotFoundErr entities.ErrEntityNotFound
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

	if errors.Is(err, entities.ErrS3ServiceDisabled) {
		log.Warn("s3 service is disabled")
		return NewError(Gone, err.Error())
	}

	if errors.Is(err, entities.ErrAuthServiceDisabled) {
		log.Warn("auth service is disabled")
		return NewError(Gone, err.Error())
	}

	if errors.Is(err, entities.ErrRoutingServiceDisabled) {
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
	GetAll(ctx context.Context, query entities.Query) ([]*T, int64, error)
	GetByID(ctx context.Context, id int32) (*T, error)
	Create(ctx context.Context, createData *CreateType) (*T, error)
	Update(ctx context.Context, id int32, updateData *UpdateType) (*T, error)
	Delete(ctx context.Context, id int32) error
}

type InfoService interface {
	Service
	GetAppInfo(context.Context) (*entities.App, error)
	GetAppInfoResponse(context.Context) (*entities.App, error)
	GetMapInfo(context.Context) (*entities.Map, error)
	GetServerInfo(context.Context) (*entities.Server, error)
	GetServices(context.Context) (*entities.Services, error)
	GetStatistics(context.Context) (*entities.DataStatistics, error)
}

type TreeService interface {
	Service
	GetAll(ctx context.Context, query entities.TreeQuery) ([]*entities.Tree, int64, error)
	GetByID(ctx context.Context, id int32) (*entities.Tree, error)
	Create(ctx context.Context, createData *entities.TreeCreate) (*entities.Tree, error)
	Update(ctx context.Context, id int32, updateData *entities.TreeUpdate) (*entities.Tree, error)
	Delete(ctx context.Context, id int32) error

	GetBySensorID(ctx context.Context, id entities.SensorID) (*entities.Tree, error)
	GetNearestTrees(ctx context.Context, coord entities.Coordinate, limit int32) ([]*entities.TreeWithDistance, error)
	HandleNewSensorData(context.Context, *entities.EventNewSensorData) error
	UpdateWateringStatuses(ctx context.Context) error
	GetPlantingYears(ctx context.Context) ([]int32, error)
}

type EvaluationService interface {
	Service
	GetEvaluation(ctx context.Context) (*entities.Evaluation, error)
}

type AuthService interface {
	Service
	LoginRequest(ctx context.Context, loginRequest *entities.LoginRequest) *entities.LoginResp
	LogoutRequest(ctx context.Context, logoutRequest *entities.Logout) error
	ClientTokenCallback(ctx context.Context, loginCallback *entities.LoginCallback) (*entities.ClientToken, error)
	Register(ctx context.Context, user *entities.RegisterUser) (*entities.User, error)
	RetrospectToken(ctx context.Context, token string) (*entities.IntroSpectTokenResult, error)
	RefreshToken(ctx context.Context, refreshToken string) (*entities.ClientToken, error)
	GetAll(ctx context.Context) ([]*entities.User, error)
	GetByIDs(ctx context.Context, ids []string) ([]*entities.User, error)
	GetAllByRole(ctx context.Context, role entities.UserRole) ([]*entities.User, error)
}

type RegionService interface {
	Service
	GetAll(ctx context.Context) ([]*entities.Region, int64, error)
	GetByID(ctx context.Context, id int32) (*entities.Region, error)
}

type TreeClusterService interface {
	Service
	// TODO: use CrudService as soon as every service has pagination
	// CrudService[entities.TreeCluster, entities.TreeClusterCreate, entities.TreeClusterUpdate]
	GetAll(ctx context.Context, query entities.TreeClusterQuery) ([]*entities.TreeCluster, int64, error)
	GetByID(ctx context.Context, id int32) (*entities.TreeCluster, error)
	Create(ctx context.Context, createData *entities.TreeClusterCreate) (*entities.TreeCluster, error)
	Update(ctx context.Context, id int32, updateData *entities.TreeClusterUpdate) (*entities.TreeCluster, error)
	Delete(ctx context.Context, id int32) error

	HandleUpdateTree(context.Context, *entities.EventUpdateTree) error
	HandleCreateTree(context.Context, *entities.EventCreateTree) error
	HandleDeleteTree(context.Context, *entities.EventDeleteTree) error
	HandleNewSensorData(context.Context, *entities.EventNewSensorData) error
	HandleUpdateWateringPlan(context.Context, *entities.EventUpdateWateringPlan) error
	UpdateWateringStatuses(ctx context.Context) error
}

type SensorService interface {
	Service
	GetAll(ctx context.Context, query entities.Query) ([]*entities.Sensor, int64, error)
	GetByID(ctx context.Context, id entities.SensorID) (*entities.Sensor, error)
	Create(ctx context.Context, createData *entities.SensorCreate) (*entities.Sensor, error)
	Update(ctx context.Context, id entities.SensorID, updateData *entities.SensorUpdate) (*entities.Sensor, error)
	Delete(ctx context.Context, id entities.SensorID) error
	GetAllDataByID(ctx context.Context, id entities.SensorID) ([]*entities.SensorData, error)
	HandleMessage(ctx context.Context, payload *entities.MqttPayload) (*entities.SensorData, error)
	MapSensorToTree(ctx context.Context, sen *entities.Sensor) error
	UpdateStatuses(ctx context.Context) error
}

type CrudService[T any, CreateType any, UpdateType any] interface {
	Service
	BasicCrudService[T, CreateType, UpdateType]
}

type VehicleService interface {
	Service
	GetAll(ctx context.Context, query entities.VehicleQuery) ([]*entities.Vehicle, int64, error)
	GetAllArchived(ctx context.Context) ([]*entities.Vehicle, error)
	GetByID(ctx context.Context, id int32) (*entities.Vehicle, error)
	Create(ctx context.Context, createData *entities.VehicleCreate) (*entities.Vehicle, error)
	Update(ctx context.Context, id int32, updateData *entities.VehicleUpdate) (*entities.Vehicle, error)
	Delete(ctx context.Context, id int32) error
	Archive(ctx context.Context, id int32) error
	GetByPlate(ctx context.Context, plate string) (*entities.Vehicle, error)
}

type WateringPlanService interface {
	Service
	GetAll(ctx context.Context, query entities.Query) ([]*entities.WateringPlan, int64, error)
	GetByID(ctx context.Context, id int32) (*entities.WateringPlan, error)
	Create(ctx context.Context, createData *entities.WateringPlanCreate) (*entities.WateringPlan, error)
	Update(ctx context.Context, id int32, updateData *entities.WateringPlanUpdate) (*entities.WateringPlan, error)
	Delete(ctx context.Context, id int32) error

	PreviewRoute(ctx context.Context, transporterID int32, trailerID *int32, clusterIDs []int32) (*entities.GeoJSON, error)
	GetGPXFileStream(ctx context.Context, objName string) (io.ReadSeekCloser, error)

	UpdateStatuses(ctx context.Context) error
}

type PluginService interface {
	Service
	Register(ctx context.Context, plugin *entities.Plugin) (*entities.ClientToken, error)
	RefreshToken(ctx context.Context, auth *entities.AuthPlugin, slug string) (*entities.ClientToken, error)
	Get(ctx context.Context, slug string) (entities.Plugin, error)
	GetAll(ctx context.Context) ([]entities.Plugin, []time.Time)
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
