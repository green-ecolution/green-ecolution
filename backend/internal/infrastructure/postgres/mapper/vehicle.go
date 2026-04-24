package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/evaluation"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalVehicleRepoMapper interface {
	FromSql(src *sqlc.Vehicle) (*vehicle.Vehicle, error)
	FromSqlList(src []*sqlc.Vehicle) ([]*vehicle.Vehicle, error)
	FromSqlVehicleWithCount(src *sqlc.GetAllVehiclesWithWateringPlanCountRow) (*evaluation.VehicleEvaluation, error)
	FromSqlListVehicleWithCount(src []*sqlc.GetAllVehiclesWithWateringPlanCountRow) ([]*evaluation.VehicleEvaluation, error)
}

type InternalVehicleRepoMapperImpl struct{}

func (c *InternalVehicleRepoMapperImpl) FromSql(source *sqlc.Vehicle) (*vehicle.Vehicle, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	return &vehicle.Vehicle{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		ArchivedAt:     utils.TimePtrToTime(source.ArchivedAt),
		NumberPlate:    source.NumberPlate,
		Description:    source.Description,
		WaterCapacity:  shared.MustNewWaterCapacity(source.WaterCapacity),
		Status:         MapVehicleStatus(source.Status),
		Type:           MapVehicleType(source.Type),
		Model:          source.Model,
		DrivingLicense: MapDrivingLicense(source.DrivingLicense),
		Height:         source.Height,
		Width:          source.Width,
		Length:         source.Length,
		Weight:         source.Weight,
		Provider:       utils.StringPtrToString(source.Provider),
		AdditionalInfo: additionalInfo,
	}, nil
}

func (c *InternalVehicleRepoMapperImpl) FromSqlList(source []*sqlc.Vehicle) ([]*vehicle.Vehicle, error) {
	return utils.MapSliceErr(source, c.FromSql)
}

func (c *InternalVehicleRepoMapperImpl) FromSqlVehicleWithCount(source *sqlc.GetAllVehiclesWithWateringPlanCountRow) (*evaluation.VehicleEvaluation, error) {
	if source == nil {
		return nil, nil
	}
	return &evaluation.VehicleEvaluation{
		NumberPlate:       source.NumberPlate,
		WateringPlanCount: source.WateringPlanCount,
	}, nil
}

func (c *InternalVehicleRepoMapperImpl) FromSqlListVehicleWithCount(source []*sqlc.GetAllVehiclesWithWateringPlanCountRow) ([]*evaluation.VehicleEvaluation, error) {
	return utils.MapSliceErr(source, c.FromSqlVehicleWithCount)
}

func MapVehicleStatus(vehicleStatus sqlc.VehicleStatus) vehicle.VehicleStatus {
	return vehicle.VehicleStatus(vehicleStatus)
}

func MapVehicleType(vehicleType sqlc.VehicleType) vehicle.VehicleType {
	return vehicle.VehicleType(vehicleType)
}

func MapDrivingLicense(drivingLicense sqlc.DrivingLicense) vehicle.DrivingLicense {
	return vehicle.DrivingLicense(drivingLicense)
}
