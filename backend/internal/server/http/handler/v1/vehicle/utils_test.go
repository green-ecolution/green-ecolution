package vehicle_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
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

	TestVehicleRequest = &entities.VehicleCreate{
		NumberPlate:   "FL TBZ 123",
		Description:   "Test description",
		Status:        entities.VehicleStatusActive,
		Type:          entities.VehicleTypeTrailer,
		WaterCapacity: 2000.5,
	}
)
