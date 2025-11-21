package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeToTime
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:TimePtrToTime
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:StringPtrToString
// goverter:extend MapDrivingLicense MapVehicleStatus MapVehicleType
type InternalVehicleRepoMapper interface {
	// goverter:map AdditionalInformations AdditionalInfo | github.com/green-ecolution/green-ecolution/backend/internal/utils:MapAdditionalInfo
	FromSql(src *sqlc.Vehicle) (*entities.Vehicle, error)
	FromSqlList(src []*sqlc.Vehicle) ([]*entities.Vehicle, error)

	FromSqlVehicleWithCount(src *sqlc.GetAllVehiclesWithWateringPlanCountRow) (*entities.VehicleEvaluation, error)
	FromSqlListVehicleWithCount(src []*sqlc.GetAllVehiclesWithWateringPlanCountRow) ([]*entities.VehicleEvaluation, error)
}

func MapVehicleStatus(vehicleStatus sqlc.VehicleStatus) entities.VehicleStatus {
	return entities.VehicleStatus(vehicleStatus)
}

func MapVehicleType(vehicleType sqlc.VehicleType) entities.VehicleType {
	return entities.VehicleType(vehicleType)
}

func MapDrivingLicense(drivingLicense sqlc.DrivingLicense) entities.DrivingLicense {
	return entities.DrivingLicense(drivingLicense)
}
