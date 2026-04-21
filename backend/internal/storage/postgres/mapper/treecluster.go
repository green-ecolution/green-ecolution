package mapper

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalTreeClusterRepoMapper interface {
	FromSql(*sqlc.TreeCluster) (*entities.TreeCluster, error)
	FromSqlList([]*sqlc.TreeCluster) ([]*entities.TreeCluster, error)
	FromSqlRegionWithCount(src *sqlc.GetAllTreeClusterRegionsWithWateringPlanCountRow) (*entities.RegionEvaluation, error)
	FromSqlRegionListWithCount(src []*sqlc.GetAllTreeClusterRegionsWithWateringPlanCountRow) ([]*entities.RegionEvaluation, error)
}

type InternalTreeClusterRepoMapperImpl struct{}

func (c *InternalTreeClusterRepoMapperImpl) FromSql(source *sqlc.TreeCluster) (*entities.TreeCluster, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	result := &entities.TreeCluster{
		ID:             source.ID,
		CreatedAt:      utils.TimeToTime(source.CreatedAt),
		UpdatedAt:      utils.TimeToTime(source.UpdatedAt),
		WateringStatus: MapWateringStatus(source.WateringStatus),
		LastWatered:    timePtrToTimePtr(source.LastWatered),
		MoistureLevel:  source.MoistureLevel,
		Address:        source.Address,
		Description:    source.Description,
		Archived:       source.Archived,
		SoilCondition:  MapSoilCondition(source.SoilCondition),
		Name:           source.Name,
		Provider:       utils.StringPtrToString(source.Provider),
		AdditionalInfo: additionalInfo,
	}
	if source.Latitude != nil {
		v := *source.Latitude
		result.Latitude = &v
	}
	if source.Longitude != nil {
		v := *source.Longitude
		result.Longitude = &v
	}
	return result, nil
}

func (c *InternalTreeClusterRepoMapperImpl) FromSqlList(source []*sqlc.TreeCluster) ([]*entities.TreeCluster, error) {
	return utils.MapSliceErr(source, c.FromSql)
}

func (c *InternalTreeClusterRepoMapperImpl) FromSqlRegionWithCount(source *sqlc.GetAllTreeClusterRegionsWithWateringPlanCountRow) (*entities.RegionEvaluation, error) {
	if source == nil {
		return nil, nil
	}
	return &entities.RegionEvaluation{
		Name:              source.Name,
		WateringPlanCount: source.WateringPlanCount,
	}, nil
}

func (c *InternalTreeClusterRepoMapperImpl) FromSqlRegionListWithCount(source []*sqlc.GetAllTreeClusterRegionsWithWateringPlanCountRow) ([]*entities.RegionEvaluation, error) {
	return utils.MapSliceErr(source, c.FromSqlRegionWithCount)
}

// timePtrToTimePtr converts *time.Time applying TimeToTime conversion.
func timePtrToTimePtr(source *time.Time) *time.Time {
	if source == nil {
		return nil
	}
	t := utils.TimeToTime(*source)
	return &t
}

func MapWateringStatus(status sqlc.WateringStatus) entities.WateringStatus {
	return entities.WateringStatus(status)
}

func MapSoilCondition(condition sqlc.TreeSoilCondition) entities.TreeSoilCondition {
	return entities.TreeSoilCondition(condition)
}
