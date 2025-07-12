package mapper

import (
	"github.com/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/backend/internal/utils:TimeToTime
// goverter:extend github.com/green-ecolution/backend/internal/utils:StringPtrToString
// goverter:extend MapWateringStatus MapSoilCondition
// goverter:ignoreMissing
type InternalTreeClusterRepoMapper interface {
	// goverter:map AdditionalInformations AdditionalInfo | github.com/green-ecolution/backend/internal/utils:MapAdditionalInfo
	FromSql(*sqlc.TreeCluster) (*entities.TreeCluster, error)
	FromSqlList([]*sqlc.TreeCluster) ([]*entities.TreeCluster, error)

	FromSqlRegionWithCount(src *sqlc.GetAllTreeClusterRegionsWithWateringPlanCountRow) (*entities.RegionEvaluation, error)
	FromSqlRegionListWithCount(src []*sqlc.GetAllTreeClusterRegionsWithWateringPlanCountRow) ([]*entities.RegionEvaluation, error)
}

func MapWateringStatus(status sqlc.WateringStatus) entities.WateringStatus {
	return entities.WateringStatus(status)
}

func MapSoilCondition(condition sqlc.TreeSoilCondition) entities.TreeSoilCondition {
	return entities.TreeSoilCondition(condition)
}
