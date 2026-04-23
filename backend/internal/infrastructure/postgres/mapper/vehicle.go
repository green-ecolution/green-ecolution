package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalVehicleRepoMapper interface {
	FromSql(src *sqlc.Vehicle) (*shared.Vehicle, error)
	FromSqlList(src []*sqlc.Vehicle) ([]*shared.Vehicle, error)
	FromSqlVehicleWithCount(src *sqlc.GetAllVehiclesWithWateringPlanCountRow) (*shared.VehicleEvaluation, error)
	FromSqlListVehicleWithCount(src []*sqlc.GetAllVehiclesWithWateringPlanCountRow) ([]*shared.VehicleEvaluation, error)
}

type InternalVehicleRepoMapperImpl struct{}

func (c *InternalVehicleRepoMapperImpl) FromSql(source *sqlc.Vehicle) (*shared.Vehicle, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	return &shared.Vehicle{
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

func (c *InternalVehicleRepoMapperImpl) FromSqlList(source []*sqlc.Vehicle) ([]*shared.Vehicle, error) {
	return utils.MapSliceErr(source, c.FromSql)
}

func (c *InternalVehicleRepoMapperImpl) FromSqlVehicleWithCount(source *sqlc.GetAllVehiclesWithWateringPlanCountRow) (*shared.VehicleEvaluation, error) {
	if source == nil {
		return nil, nil
	}
	return &shared.VehicleEvaluation{
		NumberPlate:       source.NumberPlate,
		WateringPlanCount: source.WateringPlanCount,
	}, nil
}

func (c *InternalVehicleRepoMapperImpl) FromSqlListVehicleWithCount(source []*sqlc.GetAllVehiclesWithWateringPlanCountRow) ([]*shared.VehicleEvaluation, error) {
	return utils.MapSliceErr(source, c.FromSqlVehicleWithCount)
}

func MapVehicleStatus(vehicleStatus sqlc.VehicleStatus) shared.VehicleStatus {
	return shared.VehicleStatus(vehicleStatus)
}

func MapVehicleType(vehicleType sqlc.VehicleType) shared.VehicleType {
	return shared.VehicleType(vehicleType)
}

func MapDrivingLicense(drivingLicense sqlc.DrivingLicense) shared.DrivingLicense {
	return shared.DrivingLicense(drivingLicense)
}
