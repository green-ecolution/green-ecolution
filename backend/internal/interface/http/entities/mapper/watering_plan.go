package mapper

import (
	"fmt"

	"github.com/google/uuid"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/watering"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func WateringPlanFromResponse(source *watering.WateringPlan) *entities.WateringPlanResponse {
	if source == nil {
		return nil
	}
	resp := &entities.WateringPlanResponse{
		ID:               source.ID,
		CreatedAt:        source.CreatedAt,
		UpdatedAt:        source.UpdatedAt,
		Date:             source.Date,
		Description:      source.Description,
		Status:           MapWateringPlanStatus(source.Status),
		UserIDs:          MapUUIDs(source.UserIDs),
		CancellationNote: source.CancellationNote,
		GpxURL:           source.GpxURL,
		Duration:         utils.DurationToPtrFloat64(source.Duration),
		RefillCount:      source.RefillCount,
		Provider:         source.Provider,
		AdditionalInfo:   source.AdditionalInfo,
	}
	if source.Distance != nil {
		v := source.Distance.Meters()
		resp.Distance = &v
	}
	if source.TotalWaterRequired != nil {
		v := *source.TotalWaterRequired
		resp.TotalWaterRequired = &v
	}
	if source.Evaluation != nil {
		resp.Evaluation = make([]*entities.EvaluationValue, len(source.Evaluation))
		for i, ev := range source.Evaluation {
			resp.Evaluation[i] = evaluationValueToResponse(ev)
		}
	}
	return resp
}

func WateringPlanFromResponseList(source []*watering.WateringPlan) []*entities.WateringPlanResponse {
	return utils.MapSlice(source, WateringPlanFromResponse)
}

func WateringPlanFromInListResponse(source *watering.WateringPlan) *entities.WateringPlanInListResponse {
	if source == nil {
		return nil
	}
	resp := &entities.WateringPlanInListResponse{
		ID:               source.ID,
		CreatedAt:        source.CreatedAt,
		UpdatedAt:        source.UpdatedAt,
		Date:             source.Date,
		Description:      source.Description,
		Status:           MapWateringPlanStatus(source.Status),
		UserIDs:          MapUUIDs(source.UserIDs),
		CancellationNote: source.CancellationNote,
		Provider:         source.Provider,
		AdditionalInfo:   source.AdditionalInfo,
	}
	if source.Distance != nil {
		v := source.Distance.Meters()
		resp.Distance = &v
	}
	if source.TotalWaterRequired != nil {
		v := *source.TotalWaterRequired
		resp.TotalWaterRequired = &v
	}
	return resp
}

func WateringPlanFromCreateRequest(source *entities.WateringPlanCreateRequest) (*watering.WateringPlanCreate, error) {
	if source == nil {
		return nil, nil
	}
	userIDs, err := MapUUIDReq(source.UserIDs)
	if err != nil {
		return nil, err
	}
	result := &watering.WateringPlanCreate{
		Date:           source.Date,
		Description:    source.Description,
		UserIDs:        userIDs,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
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
	return result, nil
}

func WateringPlanFromUpdateRequest(source *entities.WateringPlanUpdateRequest) (*watering.WateringPlanUpdate, error) {
	if source == nil {
		return nil, nil
	}
	userIDs, err := MapUUIDReq(source.UserIDs)
	if err != nil {
		return nil, err
	}
	result := &watering.WateringPlanUpdate{
		Date:             source.Date,
		Description:      source.Description,
		CancellationNote: source.CancellationNote,
		Status:           MapWateringPlanStatusReq(source.Status),
		UserIDs:          userIDs,
		Provider:         source.Provider,
		AdditionalInfo:   source.AdditionalInfo,
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
		result.Evaluation = make([]*watering.EvaluationValue, len(source.Evaluation))
		for i, ev := range source.Evaluation {
			result.Evaluation[i] = evaluationValueFromRequest(ev)
		}
	}
	return result, nil
}

func evaluationValueToResponse(source *watering.EvaluationValue) *entities.EvaluationValue {
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

func evaluationValueFromRequest(source *entities.EvaluationValue) *watering.EvaluationValue {
	if source == nil {
		return nil
	}
	result := &watering.EvaluationValue{
		WateringPlanID: source.WateringPlanID,
		TreeClusterID:  source.TreeClusterID,
	}
	if source.ConsumedWater != nil {
		v := *source.ConsumedWater
		result.ConsumedWater = &v
	}
	return result
}

func MapWateringPlanStatus(status watering.WateringPlanStatus) entities.WateringPlanStatus {
	return entities.WateringPlanStatus(status)
}

func MapWateringPlanStatusReq(status entities.WateringPlanStatus) watering.WateringPlanStatus {
	return watering.WateringPlanStatus(status)
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

func MapUUIDReq(userIDs []string) ([]*uuid.UUID, error) {
	mappedUserIDs := make([]*uuid.UUID, 0, len(userIDs))

	for _, userIDStr := range userIDs {
		userID, err := uuid.Parse(userIDStr)
		if err != nil {
			return nil, fmt.Errorf("invalid user ID %q: %w", userIDStr, err)
		}
		mappedUserIDs = append(mappedUserIDs, &userID)
	}

	return mappedUserIDs, nil
}
