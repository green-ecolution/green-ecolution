package mapper

import (
	"github.com/google/uuid"
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func WateringPlanFromResponse(source *domain.WateringPlan) *entities.WateringPlanResponse {
	if source == nil {
		return nil
	}
	resp := &entities.WateringPlanResponse{
		ID:               source.ID,
		CreatedAt:        utils.TimeToTime(source.CreatedAt),
		UpdatedAt:        utils.TimeToTime(source.UpdatedAt),
		Date:             utils.TimeToTime(source.Date),
		Description:      source.Description,
		Status:           MapWateringPlanStatus(source.Status),
		UserIDs:          MapUUIDs(source.UserIDs),
		Transporter:      VehicleFromResponse(source.Transporter),
		Trailer:          VehicleFromResponse(source.Trailer),
		CancellationNote: source.CancellationNote,
		GpxURL:           source.GpxURL,
		Duration:         utils.DurationToPtrFloat64(source.Duration),
		RefillCount:      source.RefillCount,
		Provider:         source.Provider,
		AdditionalInfo:   utils.MapKeyValueInterface(source.AdditionalInfo),
	}
	if source.Distance != nil {
		v := *source.Distance
		resp.Distance = &v
	}
	if source.TotalWaterRequired != nil {
		v := *source.TotalWaterRequired
		resp.TotalWaterRequired = &v
	}
	if source.TreeClusters != nil {
		resp.TreeClusters = make([]*entities.TreeClusterInListResponse, len(source.TreeClusters))
		for i, tc := range source.TreeClusters {
			resp.TreeClusters[i] = WateringPlanTreeClusterInListResponse(tc)
		}
	}
	if source.Evaluation != nil {
		resp.Evaluation = make([]*entities.EvaluationValue, len(source.Evaluation))
		for i, ev := range source.Evaluation {
			resp.Evaluation[i] = evaluationValueToResponse(ev)
		}
	}
	return resp
}

func WateringPlanFromResponseList(source []*domain.WateringPlan) []*entities.WateringPlanResponse {
	return utils.MapSlice(source, WateringPlanFromResponse)
}

func WateringPlanFromInListResponse(source *domain.WateringPlan) *entities.WateringPlanInListResponse {
	if source == nil {
		return nil
	}
	resp := &entities.WateringPlanInListResponse{
		ID:               source.ID,
		CreatedAt:        utils.TimeToTime(source.CreatedAt),
		UpdatedAt:        utils.TimeToTime(source.UpdatedAt),
		Date:             utils.TimeToTime(source.Date),
		Description:      source.Description,
		Status:           MapWateringPlanStatus(source.Status),
		UserIDs:          MapUUIDs(source.UserIDs),
		Transporter:      VehicleFromResponse(source.Transporter),
		Trailer:          VehicleFromResponse(source.Trailer),
		CancellationNote: source.CancellationNote,
		Provider:         source.Provider,
		AdditionalInfo:   utils.MapKeyValueInterface(source.AdditionalInfo),
	}
	if source.Distance != nil {
		v := *source.Distance
		resp.Distance = &v
	}
	if source.TotalWaterRequired != nil {
		v := *source.TotalWaterRequired
		resp.TotalWaterRequired = &v
	}
	if source.TreeClusters != nil {
		resp.TreeClusters = make([]*entities.TreeClusterInListResponse, len(source.TreeClusters))
		for i, tc := range source.TreeClusters {
			resp.TreeClusters[i] = WateringPlanTreeClusterInListResponse(tc)
		}
	}
	return resp
}

func WateringPlanFromCreateRequest(source *entities.WateringPlanCreateRequest) *domain.WateringPlanCreate {
	if source == nil {
		return nil
	}
	result := &domain.WateringPlanCreate{
		Date:           utils.TimeToTime(source.Date),
		Description:    source.Description,
		UserIDs:        MapUUIDReq(source.UserIDs),
		Provider:       source.Provider,
		AdditionalInfo: utils.MapKeyValueInterface(source.AdditionalInfo),
	}
	if source.TreeClusterIDs != nil {
		result.TreeClusterIDs = make([]*int32, len(source.TreeClusterIDs))
		for i, id := range source.TreeClusterIDs {
			if id != nil {
				v := *id
				result.TreeClusterIDs[i] = &v
			}
		}
	}
	if source.TransporterID != nil {
		v := *source.TransporterID
		result.TransporterID = &v
	}
	if source.TrailerID != nil {
		v := *source.TrailerID
		result.TrailerID = &v
	}
	return result
}

func WateringPlanFromUpdateRequest(source *entities.WateringPlanUpdateRequest) *domain.WateringPlanUpdate {
	if source == nil {
		return nil
	}
	result := &domain.WateringPlanUpdate{
		Date:             utils.TimeToTime(source.Date),
		Description:      source.Description,
		CancellationNote: source.CancellationNote,
		Status:           MapWateringPlanStatusReq(source.Status),
		UserIDs:          MapUUIDReq(source.UserIDs),
		Provider:         source.Provider,
		AdditionalInfo:   utils.MapKeyValueInterface(source.AdditionalInfo),
	}
	if source.TreeClusterIDs != nil {
		result.TreeClusterIDs = make([]*int32, len(source.TreeClusterIDs))
		for i, id := range source.TreeClusterIDs {
			if id != nil {
				v := *id
				result.TreeClusterIDs[i] = &v
			}
		}
	}
	if source.TransporterID != nil {
		v := *source.TransporterID
		result.TransporterID = &v
	}
	if source.TrailerID != nil {
		v := *source.TrailerID
		result.TrailerID = &v
	}
	if source.Evaluation != nil {
		result.Evaluation = make([]*domain.EvaluationValue, len(source.Evaluation))
		for i, ev := range source.Evaluation {
			result.Evaluation[i] = evaluationValueFromRequest(ev)
		}
	}
	return result
}

// WateringPlanTreeClusterInListResponse maps a TreeCluster in the context of a WateringPlan.
func WateringPlanTreeClusterInListResponse(source *domain.TreeCluster) *entities.TreeClusterInListResponse {
	return TreeClusterFromInListResponse(source)
}

func evaluationValueToResponse(source *domain.EvaluationValue) *entities.EvaluationValue {
	if source == nil {
		return nil
	}
	result := &entities.EvaluationValue{
		WateringPlanID: source.WateringPlanID,
		TreeClusterID:  source.TreeClusterID,
	}
	if source.ConsumedWater != nil {
		v := *source.ConsumedWater
		result.ConsumedWater = &v
	}
	return result
}

func evaluationValueFromRequest(source *entities.EvaluationValue) *domain.EvaluationValue {
	if source == nil {
		return nil
	}
	result := &domain.EvaluationValue{
		WateringPlanID: source.WateringPlanID,
		TreeClusterID:  source.TreeClusterID,
	}
	if source.ConsumedWater != nil {
		v := *source.ConsumedWater
		result.ConsumedWater = &v
	}
	return result
}

func MapWateringPlanStatus(status domain.WateringPlanStatus) entities.WateringPlanStatus {
	return entities.WateringPlanStatus(status)
}

func MapWateringPlanStatusReq(status entities.WateringPlanStatus) domain.WateringPlanStatus {
	return domain.WateringPlanStatus(status)
}

func MapUUIDs(source []*uuid.UUID) []*uuid.UUID {
	target := make([]*uuid.UUID, len(source))
	for i, id := range source {
		if id != nil {
			uuidCopy := *id
			target[i] = &uuidCopy
		} else {
			target[i] = nil
		}
	}
	return target
}

func MapUUIDReq(userIDs []string) []*uuid.UUID {
	mappedUserIDs := make([]*uuid.UUID, len(userIDs))

	for i, userIDStr := range userIDs {
		userID, err := uuid.Parse(userIDStr)
		if err != nil {
			mappedUserIDs[i] = nil
		} else {
			mappedUserIDs[i] = &userID
		}
	}

	return mappedUserIDs
}
