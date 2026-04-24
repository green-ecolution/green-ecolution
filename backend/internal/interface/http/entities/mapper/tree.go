package mapper

import (
	"fmt"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func TreeFromResponse(source *tree.Tree) *entities.TreeResponse {
	if source == nil {
		return nil
	}
	var sensorID *string
	if source.SensorID != nil {
		s := source.SensorID.String()
		sensorID = &s
	}

	return &entities.TreeResponse{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		TreeClusterID:  source.TreeClusterID,
		SensorID:       sensorID,
		LastWatered:    source.LastWatered,
		PlantingYear:   source.PlantingYear.Year(),
		Species:        source.Species,
		Number:         source.Number,
		Latitude:       source.Coordinate.Latitude(),
		Longitude:      source.Coordinate.Longitude(),
		WateringStatus: entities.WateringStatus(source.WateringStatus),
		Description:    source.Description,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
}

func TreeFromResponseList(source []*tree.Tree) []*entities.TreeResponse {
	return utils.MapSlice(source, TreeFromResponse)
}

type parsedTreeFields struct {
	Coordinate     shared.Coordinate
	PlantingYear   tree.PlantingYear
	SensorID       *sensor.SensorID
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
	coord, err := shared.NewCoordinate(lat, lng)
	if err != nil {
		return nil, fmt.Errorf("invalid coordinate: %w", err)
	}

	py, err := tree.NewPlantingYear(plantingYear)
	if err != nil {
		return nil, fmt.Errorf("invalid planting year: %w", err)
	}

	var sid *sensor.SensorID
	if sensorID != nil {
		s, err := sensor.NewSensorID(*sensorID)
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

func TreeFromCreateRequest(source *entities.TreeCreateRequest) (*tree.TreeCreate, error) {
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

func TreeFromUpdateRequest(source *entities.TreeUpdateRequest) (*tree.TreeUpdate, error) {
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

func (f *parsedTreeFields) toTreeCreate() *tree.TreeCreate {
	return &tree.TreeCreate{
		TreeClusterID: f.TreeClusterID, SensorID: f.SensorID,
		PlantingYear: f.PlantingYear, Species: f.Species, Number: f.Number,
		Coordinate: f.Coordinate, Description: f.Description,
		Provider: f.Provider, AdditionalInfo: f.AdditionalInfo,
	}
}

func (f *parsedTreeFields) toTreeUpdate() *tree.TreeUpdate {
	return &tree.TreeUpdate{
		TreeClusterID: f.TreeClusterID, SensorID: f.SensorID,
		PlantingYear: f.PlantingYear, Species: f.Species, Number: f.Number,
		Coordinate: f.Coordinate, Description: f.Description,
		Provider: f.Provider, AdditionalInfo: f.AdditionalInfo,
	}
}
