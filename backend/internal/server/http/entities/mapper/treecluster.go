package mapper

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func TreeClusterFromResponse(source *domain.TreeCluster) *entities.TreeClusterResponse {
	if source == nil {
		return nil
	}
	resp := &entities.TreeClusterResponse{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		WateringStatus: MapWateringStatus(source.WateringStatus),
		LastWatered:    source.LastWatered,
		MoistureLevel:  source.MoistureLevel,
		Region:         RegionFromResponse(source.Region),
		Address:        source.Address,
		Description:    source.Description,
		Archived:       source.Archived,
		SoilCondition:  MapSoilCondition(source.SoilCondition),
		Name:           source.Name,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
	if source.Latitude != nil {
		v := *source.Latitude
		resp.Latitude = &v
	}
	if source.Longitude != nil {
		v := *source.Longitude
		resp.Longitude = &v
	}
	if source.Trees != nil {
		resp.Trees = make([]*entities.TreeResponse, len(source.Trees))
		for i, t := range source.Trees {
			resp.Trees[i] = treeInClusterToResponse(t)
		}
	}
	return resp
}

func TreeClusterFromResponseList(source []*domain.TreeCluster) []*entities.TreeClusterInListResponse {
	return utils.MapSlice(source, TreeClusterFromInListResponse)
}

func TreeClusterFromInListResponse(source *domain.TreeCluster) *entities.TreeClusterInListResponse {
	if source == nil {
		return nil
	}
	resp := &entities.TreeClusterInListResponse{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		WateringStatus: MapWateringStatus(source.WateringStatus),
		LastWatered:    source.LastWatered,
		MoistureLevel:  source.MoistureLevel,
		Region:         RegionFromResponse(source.Region),
		Address:        source.Address,
		Description:    source.Description,
		Archived:       source.Archived,
		TreeIDs:        MapTreesToIDs(source.Trees),
		SoilCondition:  MapSoilCondition(source.SoilCondition),
		Name:           source.Name,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
	if source.Latitude != nil {
		v := *source.Latitude
		resp.Latitude = &v
	}
	if source.Longitude != nil {
		v := *source.Longitude
		resp.Longitude = &v
	}
	return resp
}

func TreeClusterFromCreateRequest(source *entities.TreeClusterCreateRequest) *domain.TreeClusterCreate {
	if source == nil {
		return nil
	}
	result := &domain.TreeClusterCreate{
		Address:        source.Address,
		Description:    source.Description,
		Name:           source.Name,
		SoilCondition:  MapSoilConditionReq(source.SoilCondition),
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
	if source.TreeIDs != nil {
		result.TreeIDs = make([]*int32, len(source.TreeIDs))
		for i, id := range source.TreeIDs {
			if id != nil {
				v := *id
				result.TreeIDs[i] = &v
			}
		}
	}
	return result
}

func TreeClusterFromUpdateRequest(source *entities.TreeClusterUpdateRequest) *domain.TreeClusterUpdate {
	if source == nil {
		return nil
	}
	result := &domain.TreeClusterUpdate{
		Address:        source.Address,
		Description:    source.Description,
		SoilCondition:  MapSoilConditionReq(source.SoilCondition),
		Name:           source.Name,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
	if source.TreeIDs != nil {
		result.TreeIDs = make([]*int32, len(source.TreeIDs))
		for i, id := range source.TreeIDs {
			if id != nil {
				v := *id
				result.TreeIDs[i] = &v
			}
		}
	}
	return result
}

// treeInClusterToResponse maps a tree within a cluster context.
// Different from TreeFromResponse: includes Sensor, no TreeClusterID.
func treeInClusterToResponse(source *domain.Tree) *entities.TreeResponse {
	if source == nil {
		return nil
	}
	return &entities.TreeResponse{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		Sensor:         sensorInClusterToResponse(source.Sensor),
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

func sensorInClusterToResponse(source *domain.Sensor) *entities.SensorResponse {
	if source == nil {
		return nil
	}
	return &entities.SensorResponse{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		Status:         MapSensorStatus(source.Status),
		LatestData:     sensorDataInClusterToResponse(source.LatestData),
		Latitude:       source.Latitude,
		Longitude:      source.Longitude,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
}

func sensorDataInClusterToResponse(source *domain.SensorData) *entities.SensorDataResponse {
	if source == nil {
		return nil
	}
	return &entities.SensorDataResponse{
		CreatedAt: source.CreatedAt,
		UpdatedAt: source.UpdatedAt,
	}
}

func MapWateringStatus(status domain.WateringStatus) entities.WateringStatus {
	return entities.WateringStatus(status)
}

func MapSoilCondition(condition domain.TreeSoilCondition) entities.TreeSoilCondition {
	return entities.TreeSoilCondition(condition)
}

func MapSoilConditionReq(condition entities.TreeSoilCondition) domain.TreeSoilCondition {
	return domain.TreeSoilCondition(condition)
}

func MapTreesToIDs(trees []*domain.Tree) []*int32 {
	var ids []*int32
	for _, tree := range trees {
		if tree != nil {
			ids = append(ids, &tree.ID)
		}
	}
	return ids
}
