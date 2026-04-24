package mapper

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/evaluation"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalTreeClusterRepoMapper interface {
	FromSql(*sqlc.TreeCluster) (*cluster.TreeCluster, error)
	FromSqlList([]*sqlc.TreeCluster) ([]*cluster.TreeCluster, error)
	FromSqlRegionWithCount(src *sqlc.GetAllTreeClusterRegionsWithWateringPlanCountRow) (*evaluation.RegionEvaluation, error)
	FromSqlRegionListWithCount(src []*sqlc.GetAllTreeClusterRegionsWithWateringPlanCountRow) ([]*evaluation.RegionEvaluation, error)
}

type InternalTreeClusterRepoMapperImpl struct{}

func (c *InternalTreeClusterRepoMapperImpl) FromSql(source *sqlc.TreeCluster) (*cluster.TreeCluster, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	coord, err := shared.NewCoordinateFromOptional(source.Latitude, source.Longitude)
	if err != nil {
		return nil, err
	}
	result := &cluster.TreeCluster{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		WateringStatus: MapWateringStatus(source.WateringStatus),
		LastWatered:    timePtrToTimePtr(source.LastWatered),
		MoistureLevel:  source.MoistureLevel,
		Address:        source.Address,
		Description:    source.Description,
		Archived:       source.Archived,
		Coordinate:     coord,
		SoilCondition:  MapSoilCondition(source.SoilCondition),
		Name:           source.Name,
		Provider:       utils.StringPtrToString(source.Provider),
		AdditionalInfo: additionalInfo,
	}
	return result, nil
}

func (c *InternalTreeClusterRepoMapperImpl) FromSqlList(source []*sqlc.TreeCluster) ([]*cluster.TreeCluster, error) {
	return utils.MapSliceErr(source, c.FromSql)
}

func (c *InternalTreeClusterRepoMapperImpl) FromSqlRegionWithCount(source *sqlc.GetAllTreeClusterRegionsWithWateringPlanCountRow) (*evaluation.RegionEvaluation, error) {
	if source == nil {
		return nil, nil
	}
	return &evaluation.RegionEvaluation{
		Name:              source.Name,
		WateringPlanCount: source.WateringPlanCount,
	}, nil
}

func (c *InternalTreeClusterRepoMapperImpl) FromSqlRegionListWithCount(source []*sqlc.GetAllTreeClusterRegionsWithWateringPlanCountRow) ([]*evaluation.RegionEvaluation, error) {
	return utils.MapSliceErr(source, c.FromSqlRegionWithCount)
}

// timePtrToTimePtr converts *time.Time applying TimeToTime conversion.
func timePtrToTimePtr(source *time.Time) *time.Time {
	if source == nil {
		return nil
	}
	t := *source
	return &t
}

func MapWateringStatus(status sqlc.WateringStatus) shared.WateringStatus {
	return shared.WateringStatus(status)
}

func MapSoilCondition(condition sqlc.TreeSoilCondition) cluster.TreeSoilCondition {
	return cluster.TreeSoilCondition(condition)
}
