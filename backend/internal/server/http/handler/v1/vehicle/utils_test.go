package vehicle_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
)

var (
	now = time.Now()

	TestVehicle = &entities.Vehicle{
		ID:          1,
		CreatedAt:   now,
		UpdatedAt:   now,
		NumberPlate: "FL TBZ 123",
		Description: "Test description",
		Status:      entities.VehicleStatusActive,
		Type:        entities.VehicleTypeTrailer,
	}

	TestVehicles = []*entities.Vehicle{
		TestVehicle,
		{
			ID:            2,
			CreatedAt:     now,
			UpdatedAt:     now,
			NumberPlate:   "FL TBZ 3456",
			Description:   "Test description",
			Status:        entities.VehicleStatusNotAvailable,
			Type:          entities.VehicleTypeTransporter,
			WaterCapacity: 1000.5,
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
