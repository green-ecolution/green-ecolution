package routing

import (
	"fmt"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/routing"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

func ConvertLocations(cfg *config.RoutingConfig) (*routing.GeoJSONMetadata, error) {
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

	metdadata := routing.GeoJSONMetadata{
		EndPoint:      endPoint,
		StartPoint:    startPoint,
		WateringPoint: wateringPoint,
	}

	return &metdadata, nil
}

func validateLocation(location []float64) (routing.GeoJSONLocation, error) {
	if len(location) != 2 {
		return routing.GeoJSONLocation{}, fmt.Errorf("must have exactly two elements: latitude and longitude")
	}
	coord, err := shared.NewCoordinate(location[1], location[0])
	if err != nil {
		return routing.GeoJSONLocation{}, err
	}
	return routing.GeoJSONLocation{
		Coordinate: coord,
	}, nil
}
