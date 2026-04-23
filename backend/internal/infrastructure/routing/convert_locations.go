package routing

import (
	"fmt"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

func ConvertLocations(cfg *config.RoutingConfig) (*shared.GeoJSONMetadata, error) {
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

	metdadata := shared.GeoJSONMetadata{
		EndPoint:      endPoint,
		StartPoint:    startPoint,
		WateringPoint: wateringPoint,
	}

	return &metdadata, nil
}

func validateLocation(location []float64) (shared.GeoJSONLocation, error) {
	if len(location) != 2 {
		return shared.GeoJSONLocation{}, fmt.Errorf("must have exactly two elements: latitude and longitude")
	}
	coord, err := shared.NewCoordinate(location[1], location[0])
	if err != nil {
		return shared.GeoJSONLocation{}, err
	}
	return shared.GeoJSONLocation{
		Coordinate: coord,
	}, nil
}
