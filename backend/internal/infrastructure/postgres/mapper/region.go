package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/region"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalRegionRepoMapper interface {
	FromSql(src *sqlc.Region) *region.Region
	FromSqlList(src []*sqlc.Region) []*region.Region
}

type InternalRegionRepoMapperImpl struct{}

func (c *InternalRegionRepoMapperImpl) FromSql(source *sqlc.Region) *region.Region {
	if source == nil {
		return nil
	}
	return &region.Region{
		ID:        source.ID,
		CreatedAt: source.CreatedAt,
		UpdatedAt: source.UpdatedAt,
		Name:      source.Name,
	}
}

func (c *InternalRegionRepoMapperImpl) FromSqlList(source []*sqlc.Region) []*region.Region {
	return utils.MapSlice(source, c.FromSql)
}
