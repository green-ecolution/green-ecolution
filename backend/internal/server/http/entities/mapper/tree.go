package mapper

import (
	"fmt"

	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func TreeFromResponse(source *domain.Tree) *entities.TreeResponse {
	if source == nil {
		return nil
	}
	return &entities.TreeResponse{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		TreeClusterID:  MapTreeClusterToID(source.TreeCluster),
		LastWatered:    source.LastWatered,
		PlantingYear:   source.PlantingYear.Value(),
		Species:        source.Species,
		Number:         source.Number,
		Latitude:       source.Coordinate.Latitude(),
		Longitude:      source.Coordinate.Longitude(),
		WateringStatus: MapWateringStatus(source.WateringStatus),
		Description:    source.Description,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
}

func TreeFromResponseList(source []*domain.Tree) []*entities.TreeResponse {
	return utils.MapSlice(source, TreeFromResponse)
}

func TreeFromCreateRequest(source *entities.TreeCreateRequest) (*domain.TreeCreate, error) {
	if source == nil {
		return nil, nil
	}

	coord, err := domain.NewCoordinate(source.Latitude, source.Longitude)
	if err != nil {
		return nil, fmt.Errorf("invalid coordinate: %w", err)
	}

	plantingYear, err := domain.NewPlantingYear(source.PlantingYear)
	if err != nil {
		return nil, fmt.Errorf("invalid planting year: %w", err)
	}

	result := &domain.TreeCreate{
		PlantingYear:   plantingYear,
		Species:        source.Species,
		Number:         source.Number,
		Coordinate:     coord,
		Description:    source.Description,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
	if source.TreeClusterID != nil {
		v := *source.TreeClusterID
		result.TreeClusterID = &v
	}
	if source.SensorID != nil {
		sensorID, err := domain.NewSensorID(*source.SensorID)
		if err != nil {
			return nil, fmt.Errorf("invalid sensor ID: %w", err)
		}
		result.SensorID = &sensorID
	}
	return result, nil
}

func TreeFromUpdateRequest(source *entities.TreeUpdateRequest) (*domain.TreeUpdate, error) {
	if source == nil {
		return nil, nil
	}

	coord, err := domain.NewCoordinate(source.Latitude, source.Longitude)
	if err != nil {
		return nil, fmt.Errorf("invalid coordinate: %w", err)
	}

	plantingYear, err := domain.NewPlantingYear(source.PlantingYear)
	if err != nil {
		return nil, fmt.Errorf("invalid planting year: %w", err)
	}

	result := &domain.TreeUpdate{
		PlantingYear:   plantingYear,
		Species:        source.Species,
		Number:         source.Number,
		Coordinate:     coord,
		Description:    source.Description,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
	if source.TreeClusterID != nil {
		v := *source.TreeClusterID
		result.TreeClusterID = &v
	}
	if source.SensorID != nil {
		sensorID, err := domain.NewSensorID(*source.SensorID)
		if err != nil {
			return nil, fmt.Errorf("invalid sensor ID: %w", err)
		}
		result.SensorID = &sensorID
	}
	return result, nil
}

func MapTreeClusterToID(treeCluster *domain.TreeCluster) *int32 {
	if treeCluster == nil {
		return nil
	}
	return &treeCluster.ID
}
