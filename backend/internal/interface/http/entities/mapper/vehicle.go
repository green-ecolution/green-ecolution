package mapper

import (
	"fmt"

	domain "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func VehicleFromResponse(source *domain.Vehicle) *entities.VehicleResponse {
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

func VehicleFromResponseList(source []*domain.Vehicle) []*entities.VehicleResponse {
	return utils.MapSlice(source, VehicleFromResponse)
}

func VehicleFromCreateRequest(source *entities.VehicleCreateRequest) (*domain.VehicleCreate, error) {
	if source == nil {
		return nil, nil
	}

	waterCapacity, err := domain.NewWaterCapacity(source.WaterCapacity)
	if err != nil {
		return nil, fmt.Errorf("invalid water capacity: %w", err)
	}

	return &domain.VehicleCreate{
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

func VehicleFromUpdateRequest(source *entities.VehicleUpdateRequest) (*domain.VehicleUpdate, error) {
	if source == nil {
		return nil, nil
	}

	waterCapacity, err := domain.NewWaterCapacity(source.WaterCapacity)
	if err != nil {
		return nil, fmt.Errorf("invalid water capacity: %w", err)
	}

	return &domain.VehicleUpdate{
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
