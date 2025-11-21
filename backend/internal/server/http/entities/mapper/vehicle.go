package mapper

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeToTime
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeToTimePtr
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeToPtrTime
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:MapKeyValueInterface
// goverter:extend MapVehicleStatus MapVehicleType MapVehicleStatusReq MapVehicleTypeReq MapDrivingLicense MapDrivingLicenseReq
type VehicleHTTPMapper interface {
	FromResponse(*domain.Vehicle) *entities.VehicleResponse
	FromResponseList([]*domain.Vehicle) []*entities.VehicleResponse
	FromCreateRequest(*entities.VehicleCreateRequest) *domain.VehicleCreate
	FromUpdateRequest(*entities.VehicleUpdateRequest) *domain.VehicleUpdate
}

func MapVehicleStatus(vehicleStatus domain.VehicleStatus) entities.VehicleStatus {
	return entities.VehicleStatus(vehicleStatus)
}

func MapVehicleType(vehicleType domain.VehicleType) entities.VehicleType {
	return entities.VehicleType(vehicleType)
}

func MapDrivingLicense(drivingLicense domain.DrivingLicense) entities.DrivingLicense {
	return entities.DrivingLicense(drivingLicense)
}

func MapVehicleStatusReq(vehicleStatus entities.VehicleStatus) domain.VehicleStatus {
	return domain.VehicleStatus(vehicleStatus)
}

func MapVehicleTypeReq(vehicleType entities.VehicleType) domain.VehicleType {
	return domain.VehicleType(vehicleType)
}

func MapDrivingLicenseReq(drivingLicense entities.DrivingLicense) domain.DrivingLicense {
	return domain.DrivingLicense(drivingLicense)
}
