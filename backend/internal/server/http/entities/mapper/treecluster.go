package mapper

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeToTime
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeToTimePtr
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:MapKeyValueInterface
// goverter:extend MapWateringStatus MapSoilCondition MapSoilConditionReq MapTreesToIDs MapSensorStatus
// goverter:ignoreMissing
type TreeClusterHTTPMapper interface {
	FromResponse(*domain.TreeCluster) *entities.TreeClusterResponse
	FromResponseList([]*domain.TreeCluster) []*entities.TreeClusterInListResponse
	FromCreateRequest(*entities.TreeClusterCreateRequest) *domain.TreeClusterCreate
	FromUpdateRequest(*entities.TreeClusterUpdateRequest) *domain.TreeClusterUpdate

	// goverter:map Trees TreeIDs
	FromInListResponse(*domain.TreeCluster) *entities.TreeClusterInListResponse
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
