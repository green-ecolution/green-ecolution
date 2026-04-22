package mapper

import (
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
		PlantingYear:   source.PlantingYear,
		Species:        source.Species,
		Number:         source.Number,
		Latitude:       source.Latitude,
		Longitude:      source.Longitude,
		WateringStatus: MapWateringStatus(source.WateringStatus),
		Description:    source.Description,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
}

func TreeFromResponseList(source []*domain.Tree) []*entities.TreeResponse {
	return utils.MapSlice(source, TreeFromResponse)
}

func TreeFromCreateRequest(source *entities.TreeCreateRequest) *domain.TreeCreate {
	if source == nil {
		return nil
	}
	result := &domain.TreeCreate{
		PlantingYear:   source.PlantingYear,
		Species:        source.Species,
		Number:         source.Number,
		Latitude:       source.Latitude,
		Longitude:      source.Longitude,
		Description:    source.Description,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
	if source.TreeClusterID != nil {
		v := *source.TreeClusterID
		result.TreeClusterID = &v
	}
	if source.SensorID != nil {
		v := *source.SensorID
		result.SensorID = &v
	}
	return result
}

func TreeFromUpdateRequest(source *entities.TreeUpdateRequest) *domain.TreeUpdate {
	if source == nil {
		return nil
	}
	result := &domain.TreeUpdate{
		PlantingYear:   source.PlantingYear,
		Species:        source.Species,
		Number:         source.Number,
		Latitude:       source.Latitude,
		Longitude:      source.Longitude,
		Description:    source.Description,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
	if source.TreeClusterID != nil {
		v := *source.TreeClusterID
		result.TreeClusterID = &v
	}
	if source.SensorID != nil {
		v := *source.SensorID
		result.SensorID = &v
	}
	return result
}

func MapTreeClusterToID(treeCluster *domain.TreeCluster) *int32 {
	if treeCluster == nil {
		return nil
	}
	return &treeCluster.ID
}
