package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalVehicleRepoMapper interface {
	FromSql(src *sqlc.Vehicle) (*entities.Vehicle, error)
	FromSqlList(src []*sqlc.Vehicle) ([]*entities.Vehicle, error)
	FromSqlVehicleWithCount(src *sqlc.GetAllVehiclesWithWateringPlanCountRow) (*entities.VehicleEvaluation, error)
	FromSqlListVehicleWithCount(src []*sqlc.GetAllVehiclesWithWateringPlanCountRow) ([]*entities.VehicleEvaluation, error)
}

type InternalVehicleRepoMapperImpl struct{}

func (c *InternalVehicleRepoMapperImpl) FromSql(source *sqlc.Vehicle) (*entities.Vehicle, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	return &entities.Vehicle{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		ArchivedAt:     utils.TimePtrToTime(source.ArchivedAt),
		NumberPlate:    source.NumberPlate,
		Description:    source.Description,
		WaterCapacity:  entities.MustNewWaterCapacity(source.WaterCapacity),
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

func (c *InternalVehicleRepoMapperImpl) FromSqlList(source []*sqlc.Vehicle) ([]*entities.Vehicle, error) {
	return utils.MapSliceErr(source, c.FromSql)
}

func (c *InternalVehicleRepoMapperImpl) FromSqlVehicleWithCount(source *sqlc.GetAllVehiclesWithWateringPlanCountRow) (*entities.VehicleEvaluation, error) {
	if source == nil {
		return nil, nil
	}
	return &entities.VehicleEvaluation{
		NumberPlate:       source.NumberPlate,
		WateringPlanCount: source.WateringPlanCount,
	}, nil
}

func (c *InternalVehicleRepoMapperImpl) FromSqlListVehicleWithCount(source []*sqlc.GetAllVehiclesWithWateringPlanCountRow) ([]*entities.VehicleEvaluation, error) {
	return utils.MapSliceErr(source, c.FromSqlVehicleWithCount)
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
