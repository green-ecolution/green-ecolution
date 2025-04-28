package routing

import (
	"fmt"

	"github.com/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/backend/internal/entities"
)

func ConvertLocations(cfg *config.RoutingConfig) (*entities.GeoJSONMetadata, error) {
	endPoint, err := validateLocation(cfg.EndPoint)
	if err != nil {
		return nil, fmt.Errorf("invalid EndPoint configuration: %w", err)
	}

	startPoint, err := validateLocation(cfg.StartPoint)
	if err != nil {
		return nil, fmt.Errorf("invalid StartPoint configuration: %w", err)
	}

	wateringPoint, err := validateLocation(cfg.WateringPoint)
	if err != nil {
		return nil, fmt.Errorf("invalid WateringPoint configuration: %w", err)
	}

	metdadata := entities.GeoJSONMetadata{
		EndPoint:      endPoint,
		StartPoint:    startPoint,
		WateringPoint: wateringPoint,
	}

	return &metdadata, nil
}

func validateLocation(location []float64) (entities.GeoJSONLocation, error) {
	if len(location) != 2 {
		return entities.GeoJSONLocation{}, fmt.Errorf("must have exactly two elements: latitude and longitude")
	}
	return entities.GeoJSONLocation{
		Longitude: location[0],
		Latitude:  location[1],
	}, nil
}
