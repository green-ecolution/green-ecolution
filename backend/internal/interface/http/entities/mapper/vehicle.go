package mapper

import (
	"fmt"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func VehicleFromResponse(source *vehicle.Vehicle) *entities.VehicleResponse {
	if source == nil {
		return nil
	}
	return &entities.VehicleResponse{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		ArchivedAt:     utils.TimeToPtrTime(source.ArchivedAt),
		NumberPlate:    source.NumberPlate,
		Description:    source.Description,
		WaterCapacity:  source.WaterCapacity.Liters(),
		Status:         MapVehicleStatus(source.Status),
		Type:           MapVehicleType(source.Type),
		Model:          source.Model,
		DrivingLicense: MapDrivingLicense(source.DrivingLicense),
		Height:         source.Height,
		Width:          source.Width,
		Length:         source.Length,
		Weight:         source.Weight,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
}

func VehicleFromResponseList(source []*vehicle.Vehicle) []*entities.VehicleResponse {
	return utils.MapSlice(source, VehicleFromResponse)
}

func VehicleFromCreateRequest(source *entities.VehicleCreateRequest) (*vehicle.VehicleCreate, error) {
	if source == nil {
		return nil, nil
	}

	waterCapacity, err := shared.NewWaterCapacity(source.WaterCapacity)
	if err != nil {
		return nil, fmt.Errorf("invalid water capacity: %w", err)
	}

	return &vehicle.VehicleCreate{
		NumberPlate:    source.NumberPlate,
		Description:    source.Description,
		WaterCapacity:  waterCapacity,
		Status:         MapVehicleStatusReq(source.Status),
		Type:           MapVehicleTypeReq(source.Type),
		Model:          source.Model,
		DrivingLicense: MapDrivingLicenseReq(source.DrivingLicense),
		Height:         source.Height,
		Width:          source.Width,
		Length:         source.Length,
		Weight:         source.Weight,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}, nil
}

func VehicleFromUpdateRequest(source *entities.VehicleUpdateRequest) (*vehicle.VehicleUpdate, error) {
	if source == nil {
		return nil, nil
	}

	waterCapacity, err := shared.NewWaterCapacity(source.WaterCapacity)
	if err != nil {
		return nil, fmt.Errorf("invalid water capacity: %w", err)
	}

	return &vehicle.VehicleUpdate{
		NumberPlate:    source.NumberPlate,
		Description:    source.Description,
		WaterCapacity:  waterCapacity,
		Status:         MapVehicleStatusReq(source.Status),
		Type:           MapVehicleTypeReq(source.Type),
		Model:          source.Model,
		DrivingLicense: MapDrivingLicenseReq(source.DrivingLicense),
		Height:         source.Height,
		Width:          source.Width,
		Length:         source.Length,
		Weight:         source.Weight,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}, nil
}

func MapVehicleStatus(vehicleStatus vehicle.VehicleStatus) entities.VehicleStatus {
	return entities.VehicleStatus(vehicleStatus)
}

func MapVehicleType(vehicleType vehicle.VehicleType) entities.VehicleType {
	return entities.VehicleType(vehicleType)
}

func MapDrivingLicense(drivingLicense vehicle.DrivingLicense) entities.DrivingLicense {
	return entities.DrivingLicense(drivingLicense)
}

func MapVehicleStatusReq(vehicleStatus entities.VehicleStatus) vehicle.VehicleStatus {
	return vehicle.VehicleStatus(vehicleStatus)
}

func MapVehicleTypeReq(vehicleType entities.VehicleType) vehicle.VehicleType {
	return vehicle.VehicleType(vehicleType)
}

func MapDrivingLicenseReq(drivingLicense entities.DrivingLicense) vehicle.DrivingLicense {
	return vehicle.DrivingLicense(drivingLicense)
}
