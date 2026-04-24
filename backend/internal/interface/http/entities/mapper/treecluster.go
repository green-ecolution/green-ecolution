package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func TreeClusterFromResponse(source *cluster.TreeCluster) *entities.TreeClusterResponse {
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
		Address:        source.Address,
		Description:    source.Description,
		Archived:       source.Archived,
		SoilCondition:  MapSoilCondition(source.SoilCondition),
		Name:           source.Name,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
	if source.Coordinate != nil {
		lat := source.Coordinate.Latitude()
		lng := source.Coordinate.Longitude()
		resp.Latitude = &lat
		resp.Longitude = &lng
	}
	return resp
}

func TreeClusterFromResponseList(source []*cluster.TreeCluster) []*entities.TreeClusterInListResponse {
	return utils.MapSlice(source, TreeClusterFromInListResponse)
}

func TreeClusterFromInListResponse(source *cluster.TreeCluster) *entities.TreeClusterInListResponse {
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
		Address:        source.Address,
		Description:    source.Description,
		Archived:       source.Archived,
		TreeIDs:        mapInt32SliceToPtrSlice(source.TreeIDs),
		SoilCondition:  MapSoilCondition(source.SoilCondition),
		Name:           source.Name,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
	if source.Coordinate != nil {
		lat := source.Coordinate.Latitude()
		lng := source.Coordinate.Longitude()
		resp.Latitude = &lat
		resp.Longitude = &lng
	}
	return resp
}

func TreeClusterFromCreateRequest(source *entities.TreeClusterCreateRequest) *cluster.TreeClusterCreate {
	if source == nil {
		return nil
	}
	result := &cluster.TreeClusterCreate{
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

func TreeClusterFromUpdateRequest(source *entities.TreeClusterUpdateRequest) *cluster.TreeClusterUpdate {
	if source == nil {
		return nil
	}
	result := &cluster.TreeClusterUpdate{
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

func MapWateringStatus(status shared.WateringStatus) entities.WateringStatus {
	return entities.WateringStatus(status)
}

func MapSoilCondition(condition cluster.TreeSoilCondition) entities.TreeSoilCondition {
	return entities.TreeSoilCondition(condition)
}

func MapSoilConditionReq(condition entities.TreeSoilCondition) cluster.TreeSoilCondition {
	return cluster.TreeSoilCondition(condition)
}

func mapInt32SliceToPtrSlice(ids []int32) []*int32 {
	if ids == nil {
		return nil
	}
	result := make([]*int32, len(ids))
	for i := range ids {
		v := ids[i]
		result[i] = &v
	}
	return result
}
