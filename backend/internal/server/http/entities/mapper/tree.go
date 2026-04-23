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
		PlantingYear:   source.PlantingYear.Year(),
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

type parsedTreeFields struct {
	Coordinate     domain.Coordinate
	PlantingYear   domain.PlantingYear
	SensorID       *domain.SensorID
	TreeClusterID  *int32
	Species        string
	Number         string
	Description    string
	Provider       string
	AdditionalInfo map[string]interface{}
}

func parseTreeRequest(
	lat, lng float64,
	plantingYear int32,
	sensorID *string,
	treeClusterID *int32,
	species, number, description, provider string,
	additionalInfo map[string]interface{},
) (*parsedTreeFields, error) {
	coord, err := domain.NewCoordinate(lat, lng)
	if err != nil {
		return nil, fmt.Errorf("invalid coordinate: %w", err)
	}

	py, err := domain.NewPlantingYear(plantingYear)
	if err != nil {
		return nil, fmt.Errorf("invalid planting year: %w", err)
	}

	var sid *domain.SensorID
	if sensorID != nil {
		s, err := domain.NewSensorID(*sensorID)
		if err != nil {
			return nil, fmt.Errorf("invalid sensor ID: %w", err)
		}
		sid = &s
	}

	var tcID *int32
	if treeClusterID != nil {
		v := *treeClusterID
		tcID = &v
	}

	return &parsedTreeFields{
		Coordinate:     coord,
		PlantingYear:   py,
		SensorID:       sid,
		TreeClusterID:  tcID,
		Species:        species,
		Number:         number,
		Description:    description,
		Provider:       provider,
		AdditionalInfo: additionalInfo,
	}, nil
}

func TreeFromCreateRequest(source *entities.TreeCreateRequest) (*domain.TreeCreate, error) {
	if source == nil {
		return nil, nil
	}
	f, err := parseTreeRequest(
		source.Latitude, source.Longitude, source.PlantingYear,
		source.SensorID, source.TreeClusterID,
		source.Species, source.Number, source.Description, source.Provider,
		source.AdditionalInfo,
	)
	if err != nil {
		return nil, err
	}
	return f.toTreeCreate(), nil
}

func TreeFromUpdateRequest(source *entities.TreeUpdateRequest) (*domain.TreeUpdate, error) {
	if source == nil {
		return nil, nil
	}
	f, err := parseTreeRequest(
		source.Latitude, source.Longitude, source.PlantingYear,
		source.SensorID, source.TreeClusterID,
		source.Species, source.Number, source.Description, source.Provider,
		source.AdditionalInfo,
	)
	if err != nil {
		return nil, err
	}
	return f.toTreeUpdate(), nil
}

func (f *parsedTreeFields) toTreeCreate() *domain.TreeCreate {
	return &domain.TreeCreate{
		TreeClusterID: f.TreeClusterID, SensorID: f.SensorID,
		PlantingYear: f.PlantingYear, Species: f.Species, Number: f.Number,
		Coordinate: f.Coordinate, Description: f.Description,
		Provider: f.Provider, AdditionalInfo: f.AdditionalInfo,
	}
}

func (f *parsedTreeFields) toTreeUpdate() *domain.TreeUpdate {
	return &domain.TreeUpdate{
		TreeClusterID: f.TreeClusterID, SensorID: f.SensorID,
		PlantingYear: f.PlantingYear, Species: f.Species, Number: f.Number,
		Coordinate: f.Coordinate, Description: f.Description,
		Provider: f.Provider, AdditionalInfo: f.AdditionalInfo,
	}
}

func MapTreeClusterToID(treeCluster *domain.TreeCluster) *int32 {
	if treeCluster == nil {
		return nil
	}
	return &treeCluster.ID
}
