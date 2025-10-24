package mapper

import (
	domain "github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/server/http/entities"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/backend/internal/utils:TimeToTime
// goverter:extend github.com/green-ecolution/backend/internal/utils:MapKeyValueInterface
// goverter:extend MapTreeClusterToID MapWateringStatus
type TreeHTTPMapper interface {
	// goverter:map TreeCluster TreeClusterID
	// goverter:ignore Sensor
	FromResponse(*domain.Tree) *entities.TreeResponse
	FromResponseList([]*domain.Tree) []*entities.TreeResponse
	FromUpdateRequest(*entities.TreeUpdateRequest) *domain.TreeUpdate
	FromCreateRequest(*entities.TreeCreateRequest) *domain.TreeCreate
}

func MapTreeClusterToID(treeCluster *domain.TreeCluster) *int32 {
	if treeCluster == nil {
		return nil
	}
	return &treeCluster.ID
}
