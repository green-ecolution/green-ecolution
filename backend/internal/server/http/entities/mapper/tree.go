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

type treeRequestFields struct {
	TreeClusterID  *int32
	PlantingYear   int32
	Species        string
	Number         string
	Latitude       float64
	Longitude      float64
	SensorID       *string
	Description    string
	Provider       string
	AdditionalInfo map[string]interface{}
}

func parseTreeRequestFields(f treeRequestFields) (domain.Coordinate, domain.PlantingYear, *domain.SensorID, error) {
	coord, err := domain.NewCoordinate(f.Latitude, f.Longitude)
	if err != nil {
		return domain.Coordinate{}, domain.PlantingYear{}, nil, fmt.Errorf("invalid coordinate: %w", err)
	}

	plantingYear, err := domain.NewPlantingYear(f.PlantingYear)
	if err != nil {
		return domain.Coordinate{}, domain.PlantingYear{}, nil, fmt.Errorf("invalid planting year: %w", err)
	}

	var sensorID *domain.SensorID
	if f.SensorID != nil {
		sid, err := domain.NewSensorID(*f.SensorID)
		if err != nil {
			return domain.Coordinate{}, domain.PlantingYear{}, nil, fmt.Errorf("invalid sensor ID: %w", err)
		}
		sensorID = &sid
	}

	return coord, plantingYear, sensorID, nil
}

func TreeFromCreateRequest(source *entities.TreeCreateRequest) (*domain.TreeCreate, error) {
	if source == nil {
		return nil, nil
	}

	coord, plantingYear, sensorID, err := parseTreeRequestFields(treeRequestFields{
		TreeClusterID: source.TreeClusterID, PlantingYear: source.PlantingYear,
		Species: source.Species, Number: source.Number,
		Latitude: source.Latitude, Longitude: source.Longitude,
		SensorID: source.SensorID, Description: source.Description,
		Provider: source.Provider, AdditionalInfo: source.AdditionalInfo,
	})
	if err != nil {
		return nil, err
	}

	result := &domain.TreeCreate{
		PlantingYear:   plantingYear,
		Species:        source.Species,
		Number:         source.Number,
		Coordinate:     coord,
		Description:    source.Description,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
		SensorID:       sensorID,
	}
	if source.TreeClusterID != nil {
		v := *source.TreeClusterID
		result.TreeClusterID = &v
	}
	return result, nil
}

func TreeFromUpdateRequest(source *entities.TreeUpdateRequest) (*domain.TreeUpdate, error) {
	if source == nil {
		return nil, nil
	}

	coord, plantingYear, sensorID, err := parseTreeRequestFields(treeRequestFields{
		TreeClusterID: source.TreeClusterID, PlantingYear: source.PlantingYear,
		Species: source.Species, Number: source.Number,
		Latitude: source.Latitude, Longitude: source.Longitude,
		SensorID: source.SensorID, Description: source.Description,
		Provider: source.Provider, AdditionalInfo: source.AdditionalInfo,
	})
	if err != nil {
		return nil, err
	}

	result := &domain.TreeUpdate{
		PlantingYear:   plantingYear,
		Species:        source.Species,
		Number:         source.Number,
		Coordinate:     coord,
		Description:    source.Description,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
		SensorID:       sensorID,
	}
	if source.TreeClusterID != nil {
		v := *source.TreeClusterID
		result.TreeClusterID = &v
	}
	return result, nil
}

func MapTreeClusterToID(treeCluster *domain.TreeCluster) *int32 {
	if treeCluster == nil {
		return nil
	}
	return &treeCluster.ID
}
