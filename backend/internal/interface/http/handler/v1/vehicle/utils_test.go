package vehicle_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
)

var (
	now = time.Now()

	TestVehicle = &shared.Vehicle{
		ID:          1,
		CreatedAt:   now,
		UpdatedAt:   now,
		NumberPlate: "FL TBZ 123",
		Description: "Test description",
		Status:      shared.VehicleStatusActive,
		Type:        shared.VehicleTypeTrailer,
	}

	TestVehicles = []*shared.Vehicle{
		TestVehicle,
		{
			ID:            2,
			CreatedAt:     now,
			UpdatedAt:     now,
			NumberPlate:   "FL TBZ 3456",
			Description:   "Test description",
			Status:        shared.VehicleStatusNotAvailable,
			Type:          shared.VehicleTypeTransporter,
			WaterCapacity: shared.MustNewWaterCapacity(1000.5),
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
