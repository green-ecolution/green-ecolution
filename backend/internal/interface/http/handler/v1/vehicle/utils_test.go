package vehicle_test

import (
	"time"

	domain "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
)

var (
	now = time.Now()

	TestVehicle = &domain.Vehicle{
		ID:          1,
		CreatedAt:   now,
		UpdatedAt:   now,
		NumberPlate: "FL TBZ 123",
		Description: "Test description",
		Status:      domain.VehicleStatusActive,
		Type:        domain.VehicleTypeTrailer,
	}

	TestVehicles = []*domain.Vehicle{
		TestVehicle,
		{
			ID:            2,
			CreatedAt:     now,
			UpdatedAt:     now,
			NumberPlate:   "FL TBZ 3456",
			Description:   "Test description",
			Status:        domain.VehicleStatusNotAvailable,
			Type:          domain.VehicleTypeTransporter,
			WaterCapacity: domain.MustNewWaterCapacity(1000.5),
		},
	}

	TestVehicleRequest = &serverEntities.VehicleCreateRequest{
		NumberPlate:    "FL TBZ 123",
		Description:    "Test description",
		Status:         serverEntities.VehicleStatusActive,
		Type:           serverEntities.VehicleTypeTrailer,
		WaterCapacity:  2000.5,
		Model:          "Test Model",
		DrivingLicense: serverEntities.DrivingLicenseB,
		Height:         2.5,
		Width:          1.8,
		Length:         4.0,
		Weight:         1.5,
	}
)
